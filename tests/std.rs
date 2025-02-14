//! Tests for the `std` feature of the `header_vec` crate.
#![cfg(feature = "std")]

use header_vec::*;

#[test]
fn test_extend() {
    let mut hv = HeaderVec::new(());
    hv.extend([1, 2, 3]);
    assert_eq!(hv.as_slice(), [1, 2, 3]);
}

#[test]
fn test_extend_ref() {
    let mut hv = HeaderVec::<(), i32>::new(());
    hv.extend([&1, &2, &3]);
    assert_eq!(hv.as_slice(), [1, 2, 3]);
}

#[test]
fn test_drain() {
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let drain = hv.drain(1..4);
    assert_eq!(drain.as_slice(), [2, 3, 4]);
    drop(drain);
    assert_eq!(hv.as_slice(), [1, 5, 6]);
}

#[test]
fn test_splice_nop() {
    // drain at begin
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(0..0, []);

    assert_eq!(splice.drained_slice(), []);
    drop(splice);
    assert_eq!(hv.as_slice(), [1, 2, 3, 4, 5, 6]);

    // drain inbetween
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(3..3, []);

    assert_eq!(splice.drained_slice(), []);
    drop(splice);
    assert_eq!(hv.as_slice(), [1, 2, 3, 4, 5, 6]);

    // drain at end
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(6..6, []);

    assert_eq!(splice.drained_slice(), []);
    drop(splice);
    assert_eq!(hv.as_slice(), [1, 2, 3, 4, 5, 6]);
}

#[test]
fn test_splice_insert() {
    // drain at begin
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(..0, [-2, -1, 0]);

    assert_eq!(splice.drained_slice(), []);
    drop(splice);
    assert_eq!(hv.as_slice(), [-2, -1, 0, 1, 2, 3, 4, 5, 6]);

    // drain inbetween
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(3..3, [31, 32, 33]);

    assert_eq!(splice.drained_slice(), []);
    drop(splice);
    assert_eq!(hv.as_slice(), [1, 2, 3, 31, 32, 33, 4, 5, 6]);

    // drain at end
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(6.., [7, 8, 9]);

    assert_eq!(splice.drained_slice(), []);
    drop(splice);
    assert_eq!(hv.as_slice(), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn test_splice_remove() {
    // drain at begin
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(..2, []);

    assert_eq!(splice.drained_slice(), [1, 2]);
    drop(splice);
    assert_eq!(hv.as_slice(), [3, 4, 5, 6]);

    // drain inbetween
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(3..5, []);

    assert_eq!(splice.drained_slice(), [4, 5]);
    drop(splice);
    assert_eq!(hv.as_slice(), [1, 2, 3, 6]);

    // drain at end
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(4.., []);

    assert_eq!(splice.drained_slice(), [5, 6]);
    drop(splice);
    assert_eq!(hv.as_slice(), [1, 2, 3, 4]);

    // drain all
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(.., []);

    assert_eq!(splice.drained_slice(), [1, 2, 3, 4, 5, 6]);
    drop(splice);
    assert_eq!(hv.as_slice(), []);
}

#[test]
fn test_splice_replace() {
    // same length
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(3..5, [44, 55]);

    assert_eq!(splice.drained_slice(), [4, 5]);
    drop(splice);
    assert_eq!(hv.as_slice(), [1, 2, 3, 44, 55, 6]);

    // shorter
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(3..5, [44]);

    assert_eq!(splice.drained_slice(), [4, 5]);
    drop(splice);
    assert_eq!(hv.as_slice(), [1, 2, 3, 44, 6]);

    // longer
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(4..5, [44, 55]);

    assert_eq!(splice.drained_slice(), [5]);
    drop(splice);
    assert_eq!(hv.as_slice(), [1, 2, 3, 4, 44, 55, 6]);

    // longer than tail
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(3..5, [44, 55, 56, 57, 58, 59]);

    assert_eq!(splice.drained_slice(), [4, 5]);
    drop(splice);
    assert_eq!(hv.as_slice(), [1, 2, 3, 44, 55, 56, 57, 58, 59, 6]);

    // all
    let mut hv = HeaderVec::from_header_slice((), [1, 2, 3, 4, 5, 6]);

    let splice = hv.splice(.., [11, 22, 33]);

    assert_eq!(splice.drained_slice(), [1, 2, 3, 4, 5, 6]);
    drop(splice);
    assert_eq!(hv.as_slice(), [11, 22, 33]);
}

// #[test]
// fn test_splice_zst() {
//     // same length
//     let mut hv = HeaderVec::from_header_slice((), [(),(),(),(),(),()]);
//
//     // let splice = hv.splice(3..5, [(),()]);
//     //
//     // assert_eq!(splice.drained_slice(), [(),()]);
//     // drop(splice);
//     // assert_eq!(hv.as_slice(), [(),(),(),(),(),()]);
// }
