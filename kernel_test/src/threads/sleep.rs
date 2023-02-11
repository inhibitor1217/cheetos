extern crate alloc;

type Arc<T> = alloc::sync::Arc<T>;
type Vec<T> = alloc::vec::Vec<T>;
type Mutex<T> = kernel::threads::Mutex<T>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct SleepThreadId(usize);

/// Information about an individual thread in the test.
#[derive(Debug)]
struct SleepThread {
    /// Sleeper ID.
    id: SleepThreadId,

    /// Number of ticks to sleep.
    duration: usize,

    /// Iterations counted so far.
    iterations: usize,
}

/// Returns `thread_cnt` threads sleep `iterations` times each.
pub fn sleep(test_name: &str, thread_cnt: usize, iterations: usize) {
    crate::msg!(
        test_name,
        "Creating {} threads to sleep {} times each.",
        thread_cnt,
        iterations
    );
    crate::msg!(test_name, "Thread 0 sleeps 10 ticks each time,");
    crate::msg!(test_name, "thread 1 sleeps 20 ticks each time, and so on.");
    crate::msg!(
        test_name,
        "If successful, product of each iteration count and"
    );
    crate::msg!(
        test_name,
        "sleep duration will appear in nondescending order."
    );

    let mut threads: Vec<SleepThread> = Vec::new();
    let output: Arc<Mutex<Vec<SleepThreadId>>> = Arc::new(Mutex::new(Vec::new()));
    let start_ticks = kernel::devices::timer::TIMER.lock().ticks() + 100;

    // Start threads.
    for i in 0..thread_cnt {
        let id = SleepThreadId(i);
        let duration = (i + 1) * 10;

        let thread = SleepThread {
            id,
            duration,
            iterations: 0,
        };

        threads.push(thread);

        let output = output.clone();
        kernel::threads::SCHEDULER.lock().spawn(
            move || {
                for i in 1..=iterations {
                    let sleep_until = start_ticks + i * duration;
                    let current_ticks = kernel::devices::timer::TIMER.lock().ticks();

                    kernel::threads::SCHEDULER
                        .lock()
                        .sleep(sleep_until - current_ticks);

                    output.lock().push(id);
                }
            },
            alloc::format!("thread {i}").as_str(),
            kernel::threads::thread::Thread::PRIORITY_DEFAULT,
        );
    }

    // Wait long enough for all the threads to finish.
    kernel::threads::SCHEDULER
        .lock()
        .sleep(thread_cnt * iterations * 10 + 200);

    // Print completion order.
    let mut product = 0;
    for id in output.lock().iter() {
        let thread = &mut threads[id.0];

        thread.iterations += 1;
        let new_product = thread.iterations * thread.duration;

        crate::msg!(
            test_name,
            "thread {id:?}: duration = {}, iterations = {}, product = {}",
            thread.duration,
            thread.iterations,
            new_product
        );

        if new_product >= product {
            product = new_product;
        } else {
            crate::fail!(
                test_name,
                "thread {id:?} woke up out of order ({} > {})!",
                product,
                new_product
            );
        }
    }

    // Verify that we had the proper number of wakeups.
    for thread in threads {
        if thread.iterations != iterations {
            crate::fail!(
                test_name,
                "thread {:?} woke up {} times instead of {}",
                thread.id,
                thread.iterations,
                iterations
            );
        }
    }
}
