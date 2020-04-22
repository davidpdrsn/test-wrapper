use test_wrapper::test_;

test_wrapper::setup!();

#[test_]
fn passes() {
    assert!(true);
}

#[test_]
fn fails() {
    assert!(false);
}

#[test_]
fn also_fails() {
    assert!(false);
}
