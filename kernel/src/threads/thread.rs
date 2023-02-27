extern crate alloc;

use crate::utils::data_structures::linked_list;

use super::{addr, interrupt};

/// Thread identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Id(u32);

impl Id {
    /// Returns a thread id to use for a new thread.
    pub fn new() -> Self {
        /// Atomic counter for generating [`Id`]s.
        static THREAD_ID: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(1);

        Self(THREAD_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

/// States in a thread's life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Running thread.
    Running,

    /// Not running but ready to run.
    Ready,

    /// Waiting for an event to trigger.
    Blocked,

    /// About to be destroyed.
    Dying,
}

/// A kernel thread or a user process.
///
/// Each thread structure is stored in 4 pages. The thread structure itself sits
/// at the very bottom of the stack (at offset 0). The rest of the page is
/// reserved for the thread's kernel stack, which grows downward from the top of
/// the pages (at offset 16 KiB). Here's an illustration:
///
/// ```
///   16 kB +---------------------------------+
///         |          kernel stack           |
///         |                |                |
///         |                |                |
///         |                V                |
///         |         grows downward          |
///         |                                 |
///         |                                 |
///         |                                 |
///         |                                 |
///         |                                 |
///         |                                 |
///         |                                 |
///         |                                 |
///         +---------------------------------+
///         |              magic              |
///         |                :                |
///         |                :                |
///         |              status             |
///         |                id               |
///    0 kB +---------------------------------+
/// ```
///
/// The upshot of this is twofold:
///
/// 1. First, [`Thread`] must be not allowed to grow too big. If it does, then
/// there will not be enough room for the kernel stack. Our base [`Thread`] is
/// only a few bytes in size. It probably should stay well under 1 KiB.
///
/// 2. Second, kernel stacks must not be allowed to grow too large. If a stack
/// overflows, it will corrupt the thread state. Thus, kernel functions should
/// not allocate large structures or arrays as non-static local variables. Use
/// dynamic allocation with `malloc()` or `palloc_get_page()` instead.
///
/// The first symptom of either of these problems will probably be an assertion
/// failure in [`current_thread()`], which checks that the `magic` field of the
/// running [`Thread`] is set to `Thread::MAGIC`. Stack overflow will normally
/// change this value, triggering the assertion.
#[derive(Debug)]
#[repr(C)]
pub struct Thread {
    /// Thread identifier.
    pub id: Id,

    /// Thread state.
    pub status: Status,

    /// Name (for debugging purposes).
    name: [u8; Self::NAME_LENGTH],

    /// Saved stack pointer.
    pub stack: *mut u8,

    /// Priority.
    pub priority: u32,

    /// Number of timer ticks since last yield.
    pub ticks: u32,

    /// The entrypoint function of the thread.
    entrypoint: Option<core::ptr::NonNull<dyn Fn()>>,

    /// Linked list node contained by the all-threads list of the thread
    /// scheduler.
    pub all_list_node: linked_list::Node,

    /// Linked list node contained by the ready-list of the thread scheduler,
    /// wait list of some semaphore.
    ///
    /// Shared between `thread` and `sync`.
    pub status_list_node: linked_list::Node,

    /// Detects stack overflow.
    magic: u32,
}

impl Thread {
    /// Random value for [`Thread`]'s 'magic' member.
    ///
    /// Used to detect stack overflow.
    const MAGIC: u32 = 0xcd6a_bf4b;

    /// Maximum length of a thread name.
    const NAME_LENGTH: usize = 16;

    /// Lowest priority.
    pub const PRIORITY_MIN: u32 = 0;

    /// Default priority.
    pub const PRIORITY_DEFAULT: u32 = 31;

    /// Highest priority.
    pub const PRIORITY_MAX: u32 = 63;

    /// Thread stack size; 16 KiB.
    pub const STACK_SIZE: usize = 0x4000;

    /// Number of pages in the thread stack.
    pub const STACK_PAGES: usize = Self::STACK_SIZE / addr::PAGE_SIZE;

    /// Bitmask for retrieving the [`Thread`] structure from stack pointer.
    pub const STACK_MASK: u64 = 0x3fff;

    /// Does basic initialization as a blocked thread named `name`.
    pub fn init(&mut self, name: &str, priority: u32) {
        assert!(priority <= Self::PRIORITY_MAX);
        assert!(name.len() <= Self::NAME_LENGTH);

        self.id = Id::new();
        self.status = Status::Blocked;
        self.name = [0; Self::NAME_LENGTH];
        self.name[..name.len()].copy_from_slice(name.as_bytes());
        self.stack = unsafe { (self as *mut Thread).cast::<u8>().add(Self::STACK_SIZE) };
        self.priority = priority;
        self.ticks = 0;
        self.entrypoint = None;
        self.all_list_node = linked_list::Node::new();
        self.status_list_node = linked_list::Node::new();
        self.magic = Self::MAGIC;
    }

    /// Returns true if `thread` appears to be a valid thread.
    pub fn is_thread(&self) -> bool {
        self.magic == Self::MAGIC
    }

    /// Returns the name of the thread.
    pub fn name(&self) -> &str {
        let end = self
            .name
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(Self::NAME_LENGTH);
        core::str::from_utf8(&self.name[..end]).unwrap()
    }

    /// Push `value` to the stack of the thread.
    pub fn push_to_stack<T: Sized>(&mut self, value: T) {
        assert!(core::mem::size_of::<T>() < Self::STACK_SIZE);

        unsafe {
            self.stack = self.stack.sub(core::mem::size_of::<T>());
            *(self.stack.cast::<T>()) = value;
        }
    }

    /// Configure the given closure to be the entrypoint of this thread.
    pub fn entrypoint<F>(&mut self, f: F)
    where
        F: Fn(),
        F: Send + 'static,
    {
        // Ensure `Box` is not dropped after this function terminates:
        // Perhaps, the closure should live longer than the caller thread itself.
        let entrypoint = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(f));
        self.entrypoint = unsafe { Some(core::ptr::NonNull::new_unchecked(entrypoint)) };
    }

    /// Starts the thread's main job by invoking the entrypoint.
    pub fn run(&self) {
        if let Some(entrypoint) = self.entrypoint {
            unsafe {
                (*entrypoint.as_ptr())();
            }
        }
    }
}

impl PartialEq for Thread {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Returns the running thread.
///
/// This is [`running_thread()`] plus a couple of sanity checks.
pub fn current_thread() -> &'static mut Thread {
    let thread = running_thread();

    assert!(thread.is_thread());
    assert!(thread.status == Status::Running);

    thread
}

/// Transforms the code that's currently running into a thread. This cannot work
/// in general and it is possible in this case only because the bootloader was
/// careful to put the bottom of the stack at a page boundary.
///
/// After calling this function, be sure to initialize the page allocator before
/// trying to create any threads.
///
/// It is not safe to call [`current_thread()`] until this function finishes.
pub fn init() {
    assert!(interrupt::are_disabled());

    let mut kernel_thread = running_thread();
    kernel_thread.init("main", Thread::PRIORITY_DEFAULT);
    kernel_thread.status = Status::Running;
}

/// Returns the current thread.
pub fn running_thread() -> &'static mut Thread {
    // Copy the CPU's stack pointer into `rsp`, and then round that down to the
    // stack size (16 KiB). Because `Thread` is always at the beginning of the
    // stack and the stack pointer is somewhere in the middle, this locates the
    // current `Thread`.
    let rsp = unsafe {
        let rsp: u64;
        core::arch::asm!("mov {}, rsp", out(reg) rsp);
        rsp
    };

    let thread = rsp & !Thread::STACK_MASK;
    unsafe { &mut *(thread as *mut Thread) }
}
