/// Returns `thread_cnt` threads sleep `iterations` times each.
#[macro_export]
macro_rules! test_sleep {
    ($thread_cnt:expr, $iterations:expr) => {
        kernel_test::msg!(
            "Creating {} threads to sleep {} times each.",
            $thread_cnt,
            $iterations
        );
        kernel_test::msg!("Thread 0 sleeps 10 ticks each time,");
        kernel_test::msg!("thread 1 sleeps 20 ticks each time, and so on.");
        kernel_test::msg!("If successful, product of each iteration count and");
        kernel_test::msg!("sleep duration will appear in nondescending order.");
    };
}
