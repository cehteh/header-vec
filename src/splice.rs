#![cfg(feature = "std")]

use core::{any::type_name, fmt, ptr};

use crate::{Drain, WeakFixupFn};

/// A splicing iterator for a `HeaderVec`.
///
/// This struct is created by [`Vec::splice()`].
/// See its documentation for more.
///
/// # Example
///
/// ```
/// # use header_vec::HeaderVec;
/// let mut hv: HeaderVec<(), _> = HeaderVec::from([0, 1, 2]);
/// let new = [7, 8];
/// let iter = hv.splice(1.., new);
/// ```
pub struct Splice<'a, H, I: Iterator + 'a> {
    pub(super) drain: Drain<'a, H, I::Item>,
    pub(super) replace_with: I,
    pub(super) weak_fixup: Option<WeakFixupFn<'a>>,
}

impl<H, I: Iterator> Iterator for Splice<'_, H, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.drain.size_hint()
    }
}

impl<H, I: Iterator> Splice<'_, H, I> {
    /// Not a standard function, might be useful nevertheless, we use it in tests.
    pub fn drained_slice(&self) -> &[I::Item] {
        self.drain.as_slice()
    }
}

impl<H, I: Iterator> DoubleEndedIterator for Splice<'_, H, I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.drain.next_back()
    }
}

impl<H, I: Iterator> ExactSizeIterator for Splice<'_, H, I> {}

impl<H, I> fmt::Debug for Splice<'_, H, I>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(&format!(
            "Splice<{}, {}>",
            type_name::<H>(),
            type_name::<I>()
        ))
        .field("drain", &self.drain.as_slice())
        .field("replace_with", &self.replace_with)
        .field("weak_fixup", &self.weak_fixup.is_some())
        .finish()
    }
}

impl<H, I: Iterator> Drop for Splice<'_, H, I> {
    #[track_caller]
    fn drop(&mut self) {
        self.drain.by_ref().for_each(drop);
        // At this point draining is done and the only remaining tasks are splicing
        // and moving things into the final place.
        // Which means we can replace the slice::Iter with pointers that won't point to deallocated
        // memory, so that Drain::drop is still allowed to call iter.len(), otherwise it would break
        // the ptr.sub_ptr contract.
        self.drain.iter = [].iter();

        // We will use the replace_with iterator to append elements in place on self.drain.vec.
        // When this hits the tail then elements are moved from the tail to tmp_tail.
        // When the tail is or becomes empty by that, then the remaining elements can be extended to the vec.
        //
        // Finally:
        // Then have continuous elements in the vec:  |head|replace_with|(old_tail|)spare_capacity|.
        // The old tail needs to be moved to its final destination.
        // Perhaps making space for the elements in the tmp_tail.
        let mut tmp_tail = Vec::new();

        unsafe {
            let vec = self.drain.vec.as_mut();
            loop {
                if self.drain.tail_len() == 0 {
                    // If the tail is empty, we can just extend the vector with the remaining elements.
                    // but we may have stashed some tmp_tail away and should reserve for that.
                    // PLANNED: should become 'extend_reserve()'
                    vec.reserve_intern(
                        self.replace_with.size_hint().0 + tmp_tail.len(),
                        false,
                        &mut self.weak_fixup,
                    );
                    vec.extend(self.replace_with.by_ref());
                    // in case the size_hint was not exact (or returned 0) we need to reserve for the tmp_tail
                    // in most cases this will not allocate. later we expect that we have this space reserved.
                    vec.reserve_intern(tmp_tail.len(), false, &mut self.weak_fixup);
                    break;
                } else if let Some(next) = self.replace_with.next() {
                    if vec.len_exact() >= self.drain.tail_start && self.drain.tail_len() > 0 {
                        // move one element from the tail to the tmp_tail
                        // We reserve for as much elements are hinted by replace_with or the remaining tail,
                        // whatever is smaller.
                        tmp_tail
                            .reserve(self.replace_with.size_hint().0.min(self.drain.tail_len()));
                        tmp_tail.push(ptr::read(vec.as_ptr().add(self.drain.tail_start)));
                        self.drain.tail_start += 1;
                    }

                    // since we overwrite the old tail here this will never reallocate.
                    // PLANNED: vec.push_within_capacity().unwrap_unchecked()
                    vec.push(next);
                } else {
                    // replace_with is depleted
                    break;
                }
            }

            let tail_len = self.drain.tail_len();
            if tail_len > 0 {
                // In case we need to shift the tail farther back we need to reserve space for that.
                // Reserve needs to preserve the tail we have, thus we temporarily set the length to the
                // tail_end and then restore it after the reserve.
                let old_len = vec.len_exact();
                vec.set_len(self.drain.tail_end);
                vec.reserve_intern(tmp_tail.len(), false, &mut self.weak_fixup);
                vec.set_len(old_len);

                // now we can move the tail around
                ptr::copy(
                    vec.as_ptr().add(self.drain.tail_start),
                    vec.as_mut_ptr().add(vec.len_exact() + tmp_tail.len()),
                    tail_len,
                );

                // all elements are moved from the tail, ensure that Drain drop does nothing.
                // PLANNED: eventually we may not need use Drain here
                self.drain.tail_start = self.drain.tail_end;
            }

            let tmp_tail_len = tmp_tail.len();
            if !tmp_tail.is_empty() {
                // When we stashed tail elements to tmp_tail, then fill the gap
                tmp_tail.set_len(0);
                ptr::copy_nonoverlapping(
                    tmp_tail.as_ptr(),
                    vec.as_mut_ptr().add(vec.len_exact()),
                    tmp_tail_len,
                );
            }

            // finally fix the vec length
            let new_len = vec.len_exact() + tmp_tail_len + tail_len;
            vec.set_len(new_len);
        }

        // IDEA: implement and benchmark batched copying. This leaves a gap in front of the tails which
        //       needs to be filled before resizing.
        //       Batch size:
        //       Moving one element per iteration to the tmp_tail is not efficient to make space for
        //       a element from the replace_with. Thus we determine a number of elements that we
        //       transfer in a batch to the tmp_tail. We compute the batch size to be roughly 4kb
        //       (Common page size on many systems) (or I::Item, whatever is larger) or the size of
        //       the tail when it is smaller. The later ensure that we do a single reserve with the
        //       minimum space needed when the tail is smaller than a batch would be .
        //       let batch_size = (4096 / std::mem::size_of::<I::Item>())
        //           .max(1)
        //           .min(self.drain.tail_len);
    }
}
