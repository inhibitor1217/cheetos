use crate::without_interrupts;

/// The scheduler. This module contains the implementation of the scheduler, which
/// handles the context switching and choosings of the thread to run.
#[derive(Debug)]
pub struct Scheduler {}

impl Scheduler {
    /// Creates a new scheduler.
    pub fn new() -> Self {
        Self {}
    }

    /// Puts the current thread to sleep. It will not be scheduled again awoken
    /// by [`unblock()`].
    ///
    /// This function must be called with interrupts turned off. It is usually a
    /// better idea to use one of the synchronization primitives in
    /// `threads::sync` instead.
    pub fn block_current_thread(&mut self) {
        use super::{interrupt, thread};

        assert!(!interrupt::is_external_handler_context());
        assert!(interrupt::are_disabled());

        let current = thread::current_thread();
        current.status = thread::Status::Blocked;

        self.schedule();
    }

    /// Yields the CPU. The current thread is not put to sleep and may be
    /// scheduled again immediately at the scheduler's whim.
    pub fn yield_current_thread(&mut self) {
        use super::thread;

        let current = thread::current_thread();

        without_interrupts!({
            current.status = thread::Status::Ready;
            self.schedule();
        })
    }

    /// Transitions a blocked thread to the ready-to-run state.
    /// This is an error if the thread is not blocked.
    /// (Use [`yield_current_thread()`] to make the running thread ready.)
    ///
    /// This function does not preempt the running thread. This can be
    /// important: if the caller had disabled interrupts itself, it may expect
    /// that it can atomically unblock a thread and update other data.
    pub fn unblock(&mut self, thread: &mut super::thread::Thread) {
        without_interrupts!({
            use super::thread;

            assert!(thread.is_thread());
            assert!(thread.status == thread::Status::Blocked);

            thread.status = thread::Status::Ready;
        })
    }

    /// Switches from `cur`, which must be the running thread, to `next`, which
    /// must also be running [`switch_threads()`], returning `cur` in `next`'s
    /// context.
    pub fn switch_threads(
        &mut self,
        _cur: &'static mut super::thread::Thread,
        _next: &'static mut super::thread::Thread,
    ) -> &'static mut super::thread::Thread {
        todo!()
    }

    /// Schedules a new process. At entry, interrupts must be off and the
    /// running process's state must have been changed from running to some
    /// other state. This function finds another thread to run and switches to
    /// it.
    fn schedule(&mut self) {
        use super::{interrupt, thread};

        let current = thread::current_thread();
        let next = self.next_thread_to_run();

        assert!(interrupt::are_disabled());
        assert!(current.status != thread::Status::Running);
        assert!(next.is_thread());

        if current != next {
            let _prev = self.switch_threads(current, next);
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
    fn schedule_tail(&mut self) {
        use super::{interrupt, thread};

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
    /// If the run queue is empty, then choose [`idle_thread`].
    fn next_thread_to_run(&mut self) -> &'static mut super::thread::Thread {
        use super::thread;

        // TODO: implement this properly. For now, always re-schedule the
        // current thread.
        thread::current_thread()
    }
}

/// Entrypoint of a newly created thread. This function is called when the
/// thread is first scheduled. Since [`Scheduler::switch_threads()`] only works
/// when both threads are running, we need to switch to the thread's context
/// manually.
pub fn switch_entry() {
    todo!()
}
