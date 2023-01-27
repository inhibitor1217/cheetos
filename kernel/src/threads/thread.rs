/// Thread identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ThreadId(u32);

/// A kernel thread or a user process.
///
/// Each thread structure is stored in its own 4 KiB page. The thread structure
/// itself sits at the very bottom of the page (at offset 0). The reset of the page
/// is reserved for the thread's kernel stack, which grows downward from the top of
/// the page (at offset 4 KiB). Here's an illustration:
///
/// ```
///    4 kB +---------------------------------+
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
///         |               name              |
///         |              status             |
///    0 kB +---------------------------------+
/// ```
///
/// The upshot of this is twofold:
///
/// 1. First, [`Thread`] must be not allowed to grow too big. If it does, then there will
/// not be enough room for the kernel stack. Our base [`Thread`] is only a few bytes in size.
/// It probably should stay well under 1 KiB.
///
/// 2. Second, kernel stacks must not be allowed to grow too large. If a stack overflows,
/// it will corrupt the thread state. Thus, kernel functions should not allocate large
/// structures or arrays as non-static local variables. Use dynamic allocation with `malloc()`
/// or `palloc_get_page()` instead.
///
/// The first symptom of either of these problems will probably be an assertion failure in
/// [`current_thread()`], which checks that the `magic` field of the running [`Thread`] is set to
/// `Thread::MAGIC`. Stack overflow will normally change this value, triggering the assertion.
#[derive(Debug)]
#[repr(C)]
pub struct Thread {
    /// Thread identifier.
    pub id: ThreadId,

    /// Detects stack overflow.
    pub magic: u32,
}

impl Thread {
    /// Random value for [`Thread`]'s 'magic' member.
    ///
    /// Used to detect stack overflow.
    pub const MAGIC: u32 = 0xcd6a_bf4b;
}

/// Returns the running thread.
///
/// This is [`running_thread()`] plus a couple of sanity checks.
pub fn current_thread() -> &'static mut Thread {
    running_thread()
}

/// Returns the current thread.
fn running_thread() -> &'static mut Thread {
    // Copy the CPU's stack pointer into `rsp`, and then round that down to the
    // start of the page. Because `Thread` is always at the beginning of a page
    // and the stack pointer is somewhere in the middle, this locates the current
    // `Thread`.
    let rsp: u64;
    unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) rsp);
        let rsp = x86_64::VirtAddr::new(rsp);
        &mut *x86_64::structures::paging::Page::<x86_64::structures::paging::Size4KiB>::containing_address(
            rsp,
        )
        .start_address()
        .as_mut_ptr()
    }
}

/// Transforms the code that's currently running into a thread. This cannot work
/// in general and it is possible in this case only because the bootloader was
/// careful to put the bottom of the stack at a page boundary.
///
/// After calling this function, be sure to initialize the page allocator before
/// trying to create any threads.
///
/// It is not safe to call [`current_thread()`] until this function finishes.
pub fn setup_kernel_thread() {
    todo!()
}
