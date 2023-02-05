#[test]
fn alarm_single() {
    tests_runner::run_test_kernel(env!("CARGO_BIN_FILE_TESTS_THREADS_alarm_single"));
}
