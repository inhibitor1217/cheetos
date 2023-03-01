extern crate alloc;

use core::ptr::NonNull;

use crate::{get_list_element, println, utils::data_structures::linked_list::LinkedList};

use super::{interrupt, palloc, sync, thread};

/// Stack frame for [`switch_threads()`].
#[repr(C, packed)]
struct SwitchThreadsFrame {
    rbx: usize,
    rbp: usize,
    rsi: usize,
    rdi: usize,

    /// Return address.
    rip: unsafe extern "C" fn(),
}

/// Stack frame for [`switch_entry()`].
#[repr(C, packed)]
struct SwitchEntryFrame {
    /// Return address.
    rip: unsafe extern "C" fn(),
}

/// The scheduler. This module contains the implementation of the scheduler, which
/// handles the context switching and choosings of the thread to run.
#[derive(Debug)]
pub struct Scheduler {
    /// The pointer to the idle thread, which runs when no other thread is ready
    /// to run.
    ///
    /// The idle thread is initialized when the scheduler starts.
    /// See `Scheduler::start`.
    idle_thread: Option<NonNull<thread::Thread>>,

    /// List of all threads. Threads are added to the list when they are first
    /// scheduled, and removed when they exit.
    all_list: LinkedList<thread::Thread>,

    /// List of threads in `thread::status::Ready` state, that is, threads that
    /// are ready to run but not actually running.
    ready_list: LinkedList<thread::Thread>,

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
            all_list: LinkedList::new(),
            ready_list: LinkedList::new(),
            idle_ticks: 0,
            kernel_ticks: 0,
            current_thread_ticks: 0,
        }
    }

    /// Starts a preemptive thread scheduling by enabling interrupts.
    /// Also creates the idle thread.
    pub fn start(&mut self) {
        // Add the "main" kernel thread to the all-threads list.
        self.all_list
            .push_back(&mut thread::current_thread().all_list_node);

        let idle_started = alloc::sync::Arc::new(sync::semaphore::Semaphore::new(0));
        let idle_started_clone = idle_started.clone();

        // Idle thread. Executes when no other thread is ready to run.
        //
        // The idle thread is initially put on the ready list. It will be
        // scheduled once initially, and immediately blocks. After that, the
        // idle thread never appears in the ready list. It is returned by
        // `Scheduler::next_thread_to_run` as a special case when the ready list
        // is empty.
        let idle = move || {
            idle_started_clone.up();

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

        // Wait for the idle thread to initialize.
        idle_started.down();
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
    /// Returns the new thread, or `None` if creation fails.
    ///
    /// If `spawn` has been called, then the new thread may be scheduled before
    /// `spawn` returns. Contrawise, the original thread may run for any amount
    /// of time before the new thread is scheduled. Use a semaphore or some
    /// other form of syncrhonization if you need to ensure ordering.
    ///
    /// The code provided sets the new thread's priority to `priority`, but no
    /// actual priority scheduling is implemented.
    /// Priority scheduling is the goal of Problem 1-3.
    pub fn spawn<F>(&mut self, f: F, name: &str, priority: u32) -> Option<*mut thread::Thread>
    where
        F: Fn(),
        F: Send + 'static,
    {
        // Allocate thread.
        if let Some(thread_ptr) = palloc::PAGE_ALLOCATOR
            .get_pages_aligned(
                thread::Thread::STACK_PAGES,
                thread::Thread::STACK_PAGES,
                palloc::AllocateFlags::ZERO,
            )
            .map(|page| page.start_address().as_mut_ptr::<thread::Thread>())
        {
            let thread = unsafe { &mut (*thread_ptr) };

            thread.init(name, priority);

            thread.push_to_stack(SwitchEntryFrame { rip: kernel_thread });

            thread.push_to_stack(SwitchThreadsFrame {
                rbx: 0,
                rbp: 0,
                rsi: 0,
                rdi: 0,
                rip: switch_entry,
            });

            // Set the entrypoint.
            thread.entrypoint(f);

            // Add to run queue.
            self.unblock(thread);

            Some(thread_ptr)
        } else {
            None
        }
    }

    /// Puts the current thread to sleep. It will not be scheduled again awoken
    /// by [`unblock()`].
    ///
    /// This function must be called with interrupts turned off. It is usually a
    /// better idea to use one of the synchronization primitives in
    /// `threads::sync` instead.
    pub fn block_current_thread(&mut self) {
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
        self.ready_list.push_back(&mut current.status_list_node);
        self.schedule();
    }

    /// Deschedules the current thread and destroys it.
    /// Never returns to the caller.
    pub fn exit_current_thread(&mut self) -> ! {
        assert!(!interrupt::is_external_handler_context());

        interrupt::disable();

        let current = thread::current_thread();
        current
            .all_list_node
            .cursor_mut(&mut self.all_list)
            .remove_current();
        current.status = thread::Status::Dying;

        self.schedule();

        panic!(
            "Should not reach here: thread \"{}\" should be never scheduled.",
            current.name()
        );
    }

    /// Transitions a blocked thread to the ready-to-run state.
    /// This is an error if the thread is not blocked.
    /// (Use [`yield_current_thread()`] to make the running thread ready.)
    ///
    /// This function does not preempt the running thread. This can be
    /// important: if the caller had disabled interrupts itself, it may expect
    /// that it can atomically unblock a thread and update other data.
    pub fn unblock(&mut self, thread: &'static mut thread::Thread) {
        assert!(thread.is_thread());
        assert!(thread.status == thread::Status::Blocked);

        thread.status = thread::Status::Ready;
        self.ready_list.push_back(&mut thread.status_list_node);
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
    fn schedule(&mut self) {
        let current = thread::running_thread();
        let next = self.next_thread_to_run();

        assert!(interrupt::are_disabled());
        assert!(current.status != thread::Status::Running);
        assert!(next.is_thread());

        // Perform the context switch.
        if current != next {
            unsafe {
                switch_threads(current, next);
            }
        }

        Self::schedule_tail();
    }

    /// Completes a thread switch.
    ///
    /// At this function's invocation, we just switched from the previous
    /// thread, the next thread is already running, and interrupts are still
    /// disabled. This function is normally invoked by [`schedule()`] as its
    /// final action before returning.
    ///
    /// After this function and its caller returns, the thread switch is
    /// complete.
    fn schedule_tail() {
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
    fn next_thread_to_run(&mut self) -> &'static mut thread::Thread {
        self.ready_list
            .pop_front()
            .map(|node| get_list_element!(node, thread::Thread, status_list_node))
            .or(self.idle_thread())
            .expect("Idle thread should have been initialized.")
    }

    /// Returns `true` if current thread is idle.
    fn is_idle_thread(&self) -> bool {
        let thread = thread::current_thread();

        self.idle_thread()
            .map_or_else(|| false, |idle| idle == thread)
    }

    /// Returns the idle thread, if initialized.
    fn idle_thread(&self) -> Option<&'static mut thread::Thread> {
        self.idle_thread
            .map(|thread| unsafe { &mut (*thread.as_ptr()) })
    }
}

/// The global scheduler.
///
/// It is protected behind the [`interrupt::Mutex`] to ensure
/// that only one thread can access it at a time.
pub static SCHEDULER: interrupt::Mutex<Scheduler> = interrupt::Mutex::new(Scheduler::new());

/// Function used as the basis for a kernel thread.
extern "C" fn kernel_thread() {
    interrupt::enable();

    thread::current_thread().run();

    SCHEDULER.lock().exit_current_thread();
}

core::arch::global_asm!(
    r#"
.global switch_threads
switch_threads:
    push rbx
    push rbp
    push rsi
    push rdi
    mov [rdi + 0x18], rsp
    mov rsp, [rsi + 0x18]
    call {0}
    pop rdi
    pop rsi
    pop rbp
    pop rbx
    ret
"#,
    sym reclaim_dying_thread,
);

core::arch::global_asm!(
    r#"
.global switch_entry
switch_entry:
    call {0}
    ret
"#,
    sym Scheduler::schedule_tail,
);

extern "C" {
    fn switch_threads(current: *mut thread::Thread, next: *mut thread::Thread);
    fn switch_entry();
}

/// Deallocates the thread by reclaiming the memory allocated by thread
/// and freeing the pages allocated to the thread stack.
extern "C" fn reclaim_dying_thread(thread: &'static mut thread::Thread) {
    if thread.status == thread::Status::Dying {
        // We do not deallocate main thread or idle thread.
        assert!(thread.name() != "main");
        assert!(thread.name() != "idle");
    }
}
