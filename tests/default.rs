#[test]
fn boot() {
    tests_runner::run_test_kernel(env!("CARGO_BIN_FILE_TESTS_DEFAULT_boot"));
}

#[test]
fn panic() {
    tests_runner::run_test_kernel(env!("CARGO_BIN_FILE_TESTS_DEFAULT_panic"));
}
