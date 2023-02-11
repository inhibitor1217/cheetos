#[test]
fn boot() {
    tests_runner::run_test_kernel(
        env!("CARGO_BIN_FILE_TESTS_DEFAULT_boot"),
        tests_runner::TestOptions::default(),
    );
}

#[test]
fn console() {
    tests_runner::run_test_kernel(
        env!("CARGO_BIN_FILE_TESTS_DEFAULT_console"),
        tests_runner::TestOptions::default(),
    );
}

#[test]
fn panic() {
    tests_runner::run_test_kernel(
        env!("CARGO_BIN_FILE_TESTS_DEFAULT_panic"),
        tests_runner::TestOptions::default(),
    );
}

#[test]
fn heap() {
    tests_runner::run_test_kernel(
        env!("CARGO_BIN_FILE_TESTS_DEFAULT_heap"),
        tests_runner::TestOptions::default(),
    );
}
