#[test]
fn alarm_single() {
    tests_runner::run_test_kernel(
        env!("CARGO_BIN_FILE_TESTS_THREADS_alarm_single"),
        tests_runner::TestOptions::default(),
    );
}

#[test]
fn alarm_multiple() {
    tests_runner::run_test_kernel(
        env!("CARGO_BIN_FILE_TESTS_THREADS_alarm_multiple"),
        tests_runner::TestOptions::default(),
    );
}

#[test]
fn alarm_zero() {
    tests_runner::run_test_kernel(
        env!("CARGO_BIN_FILE_TESTS_THREADS_alarm_zero"),
        tests_runner::TestOptions::default(),
    );
}
