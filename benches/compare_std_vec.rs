#![feature(test)]

extern crate std;
extern crate test;

use header_vec::*;
use test::Bencher;

#[derive(Clone, Debug, PartialEq)]
#[repr(align(128))]
struct TestA {
    a: usize,
    b: usize,
    c: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct TestAWithoutAlign {
    a: usize,
    b: usize,
    c: usize,
}

fn bench_with_header<H: Clone>(h: H, b: &mut Bencher) {
    b.iter(|| {
        let mut v = HeaderVec::<_, usize>::new(h.clone());
        const N_ELEMENTS: usize = 1000;
        for i in 0..N_ELEMENTS {
            v.push(i);
        }
        v
    });
}

#[bench]
fn test_header_vec_with_zst_create(b: &mut Bencher) {
    bench_with_header((), b);
}

#[bench]
fn test_header_vec_with_one_word_create(b: &mut Bencher) {
    bench_with_header(2usize, b);
}

#[bench]
fn test_header_vec_with_three_word_create(b: &mut Bencher) {
    bench_with_header((2usize, 2usize, 2usize), b);
}

#[bench]
fn test_header_vec_with_test_a_create(b: &mut Bencher) {
    bench_with_header(TestA { a: 1, b: 1, c: 1 }, b);
}

#[bench]
fn test_header_vec_with_test_a_without_align_create(b: &mut Bencher) {
    bench_with_header(TestAWithoutAlign { a: 1, b: 1, c: 1 }, b);
}

#[bench]
fn test_regular_vec_create(b: &mut Bencher) {
    b.iter(|| {
        let mut v = Vec::<usize>::new();
        const N_ELEMENTS: usize = 1000;
        for i in 0..N_ELEMENTS {
            v.push(i);
        }
        v
    });
}

// #[bench]
// fn test_header_vec_create_smaller(b: &mut Bencher) {
//     b.iter(|| {
//         let mut v = HeaderVec::<TestA, usize>::new(TestA { a: 4, b: !0, c: 66 });
//         const N_ELEMENTS: usize = 100;
//         for i in 0..N_ELEMENTS {
//             v.push(i);
//         }
//         v
//     });
// }

// #[bench]
// fn test_regular_vec_create_smaller(b: &mut Bencher) {
//     b.iter(|| {
//         let mut v = Vec::<usize>::new();
//         const N_ELEMENTS: usize = 100;
//         for i in 0..N_ELEMENTS {
//             v.push(i);
//         }
//         v
//     });
// }

// #[bench]
// fn test_header_vec_read(b: &mut Bencher) {
//     let mut v = HeaderVec::<TestA, usize>::new(TestA { a: 4, b: !0, c: 66 });
//     const N_ELEMENTS: usize = 1000;
//     for i in 0..N_ELEMENTS {
//         v.push(i);
//     }

//     b.iter(|| {
//         let mut acc = 0;
//         for i in 0..N_ELEMENTS {
//             acc += v[i];
//         }
//         acc
//     });
// }

// #[bench]
// fn test_regular_vec_read(b: &mut Bencher) {
//     let mut v = Vec::<usize>::new();
//     const N_ELEMENTS: usize = 1000;
//     for i in 0..N_ELEMENTS {
//         v.push(i);
//     }

//     b.iter(|| {
//         let mut acc = 0;
//         for i in 0..N_ELEMENTS {
//             acc += v[i];
//         }
//         acc
//     });
// }

#[cfg(feature = "std")]
mod stdbench {
    use super::*;
    use std::ops::{Range, RangeBounds};
    use xmacro::xmacro;

    xmacro! {
        $[
            benchfunc:      type:
            hv_drain_bench  (HeaderVec::<(), _>)
            vec_drain_bench (Vec)
        ]

        fn $benchfunc<T, R>(b: &mut Bencher, init: &[T], range: R)
        where
            T: Clone + Default,
            R: RangeBounds<usize> + Clone,
        {
            b.iter(|| {
                let mut v = $type::from(init);
                v.drain(range.clone());
                v
            });
        }
    }

    xmacro! {
        $[
            bench: init:       range:
            middle [123; 1000] (100..500)
            begin  [123; 1000] (..500)
            end    [123; 1000] (100..)
        ]

        #[bench]
        fn $+test_hv_drain_$bench(b: &mut Bencher) {
            hv_drain_bench(b, &$init, $range);
        }

        #[bench]
        fn $+test_vec_drain_$bench(b: &mut Bencher) {
            vec_drain_bench(b, &$init, $range);
        }
    }

    xmacro! {
        $[
            benchfunc:       type:
            hv_splice_bench  (HeaderVec::<(), _>)
            vec_splice_bench (Vec)
        ]

        fn $benchfunc<T, R, I>(b: &mut Bencher, init: &[T], range: R, replace_with: I)
        where
            T: Clone + Default,
            R: RangeBounds<usize> + Clone,
            I: IntoIterator<Item = T> + Clone,
        {
            b.iter(|| {
                let mut v = $type::from(init);
                v.splice(range.clone(), replace_with.clone());
                v
            });
        }
    }

    xmacro! {
        $[
            bench:         init:       range:     replace_with:
            nop            [123; 1000] (0..0)     []
            insert         [123; 1000] (100..100) [123;500]
            remove         [123; 1000] (100..600) []
            middle_shorter [123; 1000] (400..500) [234;50]
            middle_longer  [123; 1000] (400..500) [345;200]
            middle_same    [123; 1000] (400..500) [456;100]
            end_shorter    [123; 1000] (900..)    [234;50]
            end_longer     [123; 1000] (900..)    [345;200]
            end_same       [123; 1000] (900..)    [456;100]
        ]

        #[bench]
        fn $+test_hv_splice_$bench(b: &mut Bencher) {
            hv_splice_bench(b, &$init, $range, $replace_with)
        }

        #[bench]
        fn $+test_vec_splice_$bench(b: &mut Bencher) {
            vec_splice_bench(b, &$init, $range, $replace_with)
        }
    }
}
