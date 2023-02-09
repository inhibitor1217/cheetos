#[test]
fn boot() {
    tests_runner::run_test_kernel(env!("CARGO_BIN_FILE_TESTS_DEFAULT_boot"));
}

#[test]
fn console() {
    tests_runner::run_test_kernel(env!("CARGO_BIN_FILE_TESTS_DEFAULT_console"));
}

#[test]
fn panic() {
    tests_runner::run_test_kernel(env!("CARGO_BIN_FILE_TESTS_DEFAULT_panic"));
}

#[test]
fn heap() {
    tests_runner::run_test_kernel(env!("CARGO_BIN_FILE_TESTS_DEFAULT_heap"));
}
