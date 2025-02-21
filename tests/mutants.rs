use header_vec::HeaderVec;
#[test]
fn test_truncate_remaining_len() {
    let mut vec: HeaderVec<(), i32> = HeaderVec::from([1, 2, 3, 4, 5]);
    let len = vec.len();
    vec.truncate(2);
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.as_slice(), &[1, 2]);
    assert!(vec.capacity() >= len);
}

#[test]
fn test_drain_size_hint() {
    let mut vec: HeaderVec<(), i32> = HeaderVec::from([1, 2, 3, 4, 5]);
    let drain = vec.drain(1..4);
    let (lower, upper) = drain.size_hint();
    assert_eq!(lower, 3);
    assert_eq!(upper, Some(3));
}
