use testlib;

#[test]
fn test_add_function() {
    assert_eq!(testlib::add(2, 3), 5);
}

#[test]
fn test_add_zero() {
    assert_eq!(testlib::add(0, 5), 5);
    assert_eq!(testlib::add(5, 0), 5);
}

#[test]
fn test_add_negative() {
    assert_eq!(testlib::add(-2, 3), 1);
    assert_eq!(testlib::add(2, -3), -1);
}
