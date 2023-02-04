use crate::println;

use super::{control::are_disabled, mutex::Mutex, pic::PICS};

/// An interrupt handler function.
pub type InterruptHandler = fn(x86_64::structures::idt::InterruptStackFrame);

#[derive(Copy, Clone)]
struct InterruptHandlerRegistry {
    /// The handler registered for the interrupt.
    handler: Option<InterruptHandler>,

    /// Name for the interrupt, for debugging purposes.
    name: [u8; Self::NAME_LENGTH],
}

impl InterruptHandlerRegistry {
    /// Max length for interrupt names.
    const NAME_LENGTH: usize = 64;

    const fn new() -> Self {
        Self {
            handler: None,
            name: [0; Self::NAME_LENGTH],
        }
    }

    fn name(&self) -> &str {
        let end = self
            .name
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(Self::NAME_LENGTH);
        core::str::from_utf8(&self.name[..end]).unwrap()
    }

    fn set_name(&mut self, name: &str) {
        assert!(name.len() <= Self::NAME_LENGTH);
        self.name[..name.len()].copy_from_slice(name.as_bytes());
    }
}

pub struct InterruptHandlersRegistry {
    idt: x86_64::structures::idt::InterruptDescriptorTable,
    registries: [InterruptHandlerRegistry; Self::NUM_INTERRUPTS],
    is_external_context: bool,
}

impl InterruptHandlersRegistry {
    /// Number of x86 interrupts.
    const NUM_INTERRUPTS: usize = 256;

    /// Start offset of external interrupts.
    const EXTERNAL_INTERRUPT_OFFSET: usize = 0x20;

    pub const fn new() -> Self {
        Self {
            idt: x86_64::structures::idt::InterruptDescriptorTable::new(),
            registries: [InterruptHandlerRegistry::new(); Self::NUM_INTERRUPTS],
            is_external_context: false,
        }
    }

    /// Initializes the registry by loading the default interrupt handler.
    pub fn init(&mut self) {
        x86_64::set_general_handler!(&mut self.idt, interrupt_handler);

        self.registries[0].set_name("#DE Divide Error");
        self.registries[1].set_name("#DB Debug Exception");
        self.registries[2].set_name("NMI Interrupt");
        self.registries[3].set_name("#BP Breakpoint Exception");
        self.registries[4].set_name("#OF Overflow Exception");
        self.registries[5].set_name("#BR BOUND Range Exceeded Exception");
        self.registries[6].set_name("#UD Invalid Opcode Exception");
        self.registries[7].set_name("#NM Device Not Available Exception");
        self.registries[8].set_name("#DF Double Fault Exception");
        self.registries[9].set_name("Coprocessor Segment Overrun");
        self.registries[10].set_name("#TS Invalid TSS Exception");
        self.registries[11].set_name("#NP Segment Not Present");
        self.registries[12].set_name("#SS Stack Fault Exception");
        self.registries[13].set_name("#GP General Protection Exception");
        self.registries[14].set_name("#PF Page-Fault Exception");
        self.registries[16].set_name("#MF x87 FPU Floating-Point Error");
        self.registries[17].set_name("#AC Alignment Check Exception");
        self.registries[18].set_name("#MC Machine-Check Exception");
        self.registries[19].set_name("#XF SIMD Floating-Point Exception");

        for registry in &mut self.registries[Self::EXTERNAL_INTERRUPT_OFFSET..] {
            registry.set_name("unknown");
        }
    }

    /// Loads IDT register.
    pub fn load(&'static self) {
        self.idt.load();
    }

    fn is_external_context(&self) -> bool {
        self.is_external_context
    }

    fn handle_internal(
        &self,
        frame: x86_64::structures::idt::InterruptStackFrame,
        interrupt_id: u8,
    ) {
        self.handle(frame, interrupt_id);
    }

    /// External interrupts are special.
    ///
    /// We only handle one at a time, so this function must be called with
    /// interrupts disabled. An external interrupt handler cannot sleep.
    fn handle_external(
        &mut self,
        frame: x86_64::structures::idt::InterruptStackFrame,
        interrupt_id: u8,
    ) {
        assert!(are_disabled());
        assert!(!self.is_external_context);

        self.is_external_context = true;

        // Invoke the interrupt's handler.
        self.handle(frame, interrupt_id);

        // Complete the processing of an external interrupt.
        assert!(are_disabled());
        assert!(self.is_external_context());

        self.is_external_context = false;
    }

    fn handle(&self, frame: x86_64::structures::idt::InterruptStackFrame, interrupt_id: u8) {
        let registry = self.registries[interrupt_id as usize];

        if let Some(handler) = registry.handler {
            handler(frame);
        } else if interrupt_id == 0x27 || interrupt_id == 0x2f {
            // There is no handler, but this interrupt can trigger spuriously
            // due to a hardware fault or hardware race condition. Ignore it.
        } else {
            // Handle an unexpected interrupt.
            unsafe {
                println!(
                    "Unexpected interrupt {interrupt_id:#04x} {}",
                    registry.name()
                );
            }
        }
    }
}

/// A global interrupt handler registry.
pub static REGISTRY: Mutex<InterruptHandlersRegistry> =
    Mutex::new(InterruptHandlersRegistry::new());

/// Handler for internal interrupts. This function is called by the
/// interrupt handlers registered to IDT. `frame` describes the interrupt
/// and the interrupted thread's registers.
fn interrupt_handler(
    frame: x86_64::structures::idt::InterruptStackFrame,
    interrupt_id: u8,
    _error_code: Option<u64>,
) {
    let is_external = interrupt_id >= InterruptHandlersRegistry::EXTERNAL_INTERRUPT_OFFSET as u8;

    if is_external {
        REGISTRY.lock().handle_external(frame, interrupt_id);
        unsafe {
            PICS.lock().end_of_interrupt(interrupt_id);
        }
    } else {
        REGISTRY.peek().handle_internal(frame, interrupt_id);
    }
}

/// Returns `true` during processing of an external interrupt, and `false` at
/// all other times.
pub fn is_external_handler_context() -> bool {
    REGISTRY.peek().is_external_context()
}
