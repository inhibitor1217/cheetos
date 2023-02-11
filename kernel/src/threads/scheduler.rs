use core::{arch::asm, ptr::NonNull};

use crate::{devices::timer::TIMER, println};

use super::{interrupt, thread};

/// Switches from `cur`, which must be the running thread, to `next`, which
/// must also be running [`switch_threads!`], returning `cur` in `next`'s
/// context.
macro_rules! switch_threads {
    ($cur:expr, $next:expr) => {{
        unsafe {
            asm!(
                // Save caller's register state.
                //
                // Note that the SVR4 ABI alalows us to destry %rax, %rcx, %rdx,
                // but requires us to preserve %rbx, %rbp, %rsi, and %rdi.
                //
                // This stack frame must match the one set up by `Thread::new` in size.
                "push rbx",
                "push rbp",
                "push rsi",
                "push rdi",
                // Save current stack pointer to old thread's stack, if any.
                "mov {0}, rsp",
                // Restore stack pointer from new thread's stack.
                "mov rsp, {1}",
                // Restore caller's register state.
                "pop rdi",
                "pop rsi",
                "pop rbp",
                "pop rbx",
                out(reg) $cur.stack,
                in(reg) $next.stack,
            );
        }
        $cur
    }};
}

/// The scheduler. This module contains the implementation of the scheduler, which
/// handles the context switching and choosings of the thread to run.
#[derive(Debug)]
pub struct Scheduler {
    idle_thread: Option<NonNull<thread::Thread>>,

    /// Number of timer ticks spent idle.
    idle_ticks: usize,

    /// Number of timer ticks in kernel threads.
    kernel_ticks: usize,

    /// Number of timer ticks since last yield.
    current_thread_ticks: usize,
}

impl Scheduler {
    /// Number of timer ticks to give each thread.
    const TIME_SLICE: usize = 4;

    /// Creates a new scheduler.
    pub const fn new() -> Self {
        Self {
            idle_thread: None,
            idle_ticks: 0,
            kernel_ticks: 0,
            current_thread_ticks: 0,
        }
    }

    /// Starts a preemptive thread scheduling by enabling interrupts.
    /// Also creates the idle thread.
    pub fn start(&mut self) {
        // Idle thread. Executes when no other thread is ready to run.
        //
        // The idle thread is initially put on the ready list. It will be
        // scheduled once initially, and immediately blocks. After that, the
        // idle thread never appears in the ready list. It is returned by
        // `Scheduler::next_thread_to_run` as a special case when the ready list
        // is empty.
        let idle = move || {
            loop {
                // Let someone else run.
                SCHEDULER.lock().block_current_thread();

                // Wait for the next run.
                x86_64::instructions::hlt();
            }
        };

        // Create the idle thread.
        self.idle_thread = self
            .spawn(idle, "idle", thread::Thread::PRIORITY_MIN)
            .map(|thread| unsafe { NonNull::new_unchecked(thread) });

        // Start preemptive thread scheduling.
        interrupt::enable();
    }

    /// Called by the timer interrupt handler at each timer tick.
    /// Thus, this function runs in an external interrupt context.
    pub fn tick(&mut self) {
        // Update statistics.
        if self.is_idle_thread() {
            self.idle_ticks += 1;
        } else {
            self.kernel_ticks += 1;
        }
        self.current_thread_ticks += 1;
    }

    /// Creates a new kernel thread named `name` with given initial `priority`.
    ///
    /// Returns the id of the thread, or `None` if creation fails.
    ///
    /// If `spawn` has been called, then the new thread may be scheduled before
    /// `spawn` returns. Contrawise, the original thread may run for any amount
    /// of time before the new thread is scheduled. Use a semaphore or some
    /// other form of syncrhonization if you need to ensure ordering.
    ///
    /// The code provided sets the new thread's priority to `priority`, but no
    /// actual priority scheduling is implemented.
    /// Priority scheduling is the goal of Problem 1-3.
    pub fn spawn<F>(
        &mut self,
        _f: F,
        _name: &str,
        _priority: u32,
    ) -> Option<&'static mut thread::Thread>
    where
        F: FnOnce(),
        F: Send + 'static,
    {
        None
    }

    /// Puts the current thread to sleep. It will not be scheduled again awoken
    /// by [`unblock()`].
    ///
    /// This function must be called with interrupts turned off. It is usually a
    /// better idea to use one of the synchronization primitives in
    /// `threads::sync` instead.
    pub fn block_current_thread(&self) {
        assert!(!interrupt::is_external_handler_context());
        assert!(interrupt::are_disabled());

        let current = thread::current_thread();
        current.status = thread::Status::Blocked;

        self.schedule();
    }

    /// If current thread has consumed enough ticks, enforce preemption.
    pub fn preempt_current_thread(&mut self) {
        if self.current_thread_ticks >= Self::TIME_SLICE {
            self.yield_current_thread();
        }
    }

    /// Yields the CPU. The current thread is not put to sleep and may be
    /// scheduled again immediately at the scheduler's whim.
    pub fn yield_current_thread(&mut self) {
        let current = thread::current_thread();
        current.status = thread::Status::Ready;
        self.schedule();
    }

    /// Make current thread sleep for approximately `ticks` timer ticks.
    /// Interrupt must be turned on.
    pub fn sleep(&mut self, ticks: usize) {
        assert!(interrupt::are_enabled());

        let start = TIMER.lock().ticks();
        while TIMER.lock().elapsed(start) < ticks {
            self.yield_current_thread();
        }
    }

    /// Transitions a blocked thread to the ready-to-run state.
    /// This is an error if the thread is not blocked.
    /// (Use [`yield_current_thread()`] to make the running thread ready.)
    ///
    /// This function does not preempt the running thread. This can be
    /// important: if the caller had disabled interrupts itself, it may expect
    /// that it can atomically unblock a thread and update other data.
    pub fn unblock(&mut self, thread: &mut thread::Thread) {
        assert!(thread.is_thread());
        assert!(thread.status == thread::Status::Blocked);

        thread.status = thread::Status::Ready;
    }

    /// Prints thread statistics.
    pub fn print_stats(&self) {
        println!(
            "Thread: {} idle ticks, {} kernel ticks.",
            self.idle_ticks, self.kernel_ticks
        );
    }

    /// Schedules a new process. At entry, interrupts must be off and the
    /// running process's state must have been changed from running to some
    /// other state. This function finds another thread to run and switches to
    /// it.
    fn schedule(&self) {
        let current = thread::current_thread();
        let next = self.next_thread_to_run();

        assert!(interrupt::are_disabled());
        assert!(current.status != thread::Status::Running);
        assert!(next.is_thread());

        if current != next {
            let _prev = switch_threads!(current, next);
            // TODO: drop `prev` if it is dying
        }

        self.schedule_tail();
    }

    /// Completes a thread switch by activating the new thread's page tables,
    /// and, if the previous thread is dying, destroying it.
    ///
    /// At this function's invocation, we just switched from the previous
    /// thread, the next thread is already running, and interrupts are still
    /// disabled. This function is normally invoked by [`schedule()`] as its
    /// final action before returning, but the first time a thread is scheduled
    /// it is called by [`switch_entry()`].
    ///
    /// After this function and its caller returns, the thread switch is
    /// complete.
    fn schedule_tail(&self) {
        let current = thread::running_thread();

        assert!(interrupt::are_disabled());

        // Mark us as running.
        current.status = thread::Status::Running;

        // Start new time slice.
        current.ticks = 0;
    }

    /// Chooses and returns the next thread to be scheduled. Should return a
    /// thread from the run queue, unless the run queue is empty. (If the
    /// running thread can continue running, then it will be in the run queue.)
    /// If the run queue is empty, then choose `idle_thread`.
    fn next_thread_to_run(&self) -> &'static mut thread::Thread {
        // TODO: implement this properly. For now, always re-schedule the
        // current thread.
        thread::current_thread()
    }

    /// Returns `true` if current thread is idle.
    fn is_idle_thread(&self) -> bool {
        let thread = thread::current_thread();

        self.idle_thread
            .map_or_else(|| false, |idle| idle.as_ptr() == thread)
    }
}

/// The global scheduler.
///
/// It is protected behind the [`interrupt::Mutex`] to ensure
/// that only one thread can access it at a time.
pub static SCHEDULER: interrupt::Mutex<Scheduler> = interrupt::Mutex::new(Scheduler::new());
