use super::{constants::EXTERNAL_INTERRUPT_OFFSET, mutex::Mutex};

#[derive(Copy, Clone)]
struct InterruptHandlerRegistry {
    /// The handler registered for the interrupt.
    handler: Option<x86_64::structures::idt::HandlerFunc>,

    /// Name for the interrupt, for debugging purposes.
    name: [u8; Self::NAME_LENGTH],

    /// Number of unexpected invocations for this interrupt.
    /// An unexpected invocation is one that has no registered `handler`.
    unexpected_count: usize,
}

impl InterruptHandlerRegistry {
    /// Max length for interrupt names.
    const NAME_LENGTH: usize = 64;

    const fn new() -> Self {
        Self {
            handler: None,
            name: [0; Self::NAME_LENGTH],
            unexpected_count: 0,
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

    /// Maximum index of x86 interrupts.
    const MAX_INTERRUPT: usize = Self::NUM_INTERRUPTS - 1;

    pub const fn new() -> Self {
        Self {
            idt: x86_64::structures::idt::InterruptDescriptorTable::new(),
            registries: [InterruptHandlerRegistry::new(); Self::NUM_INTERRUPTS],
            is_external_context: false,
        }
    }

    /// Initializes the registry by loading the default interrupt handler.
    pub fn init(&mut self) {
        for i in 0..Self::NUM_INTERRUPTS {
            match i {
                15 | 18 | 21..=28 | 31 => (),
                8 => {
                    self.idt.double_fault.set_handler_fn(double_fault_handler);
                }
                10 => {
                    self.idt.invalid_tss.set_handler_fn(handler_with_error_code);
                }
                11 => {
                    self.idt
                        .segment_not_present
                        .set_handler_fn(handler_with_error_code);
                }
                12 => {
                    self.idt
                        .stack_segment_fault
                        .set_handler_fn(handler_with_error_code);
                }
                13 => {
                    self.idt
                        .general_protection_fault
                        .set_handler_fn(handler_with_error_code);
                }
                14 => {
                    self.idt.page_fault.set_handler_fn(page_fault_handler);
                }
                17 => {
                    self.idt
                        .alignment_check
                        .set_handler_fn(handler_with_error_code);
                }
                29 => {
                    self.idt
                        .vmm_communication_exception
                        .set_handler_fn(handler_with_error_code);
                }
                30 => {
                    self.idt
                        .security_exception
                        .set_handler_fn(handler_with_error_code);
                }
                i @ 0..=Self::MAX_INTERRUPT => {
                    self.idt[i].set_handler_fn(handler);
                }
                _ => (),
            }
        }

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

        for registry in &mut self.registries[(EXTERNAL_INTERRUPT_OFFSET as usize)..] {
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
}

/// A global interrupt handler registry.
pub static REGISTRY: Mutex<InterruptHandlersRegistry> =
    Mutex::new(InterruptHandlersRegistry::new());

extern "x86-interrupt" fn handler(_frame: x86_64::structures::idt::InterruptStackFrame) {}

extern "x86-interrupt" fn handler_with_error_code(
    _frame: x86_64::structures::idt::InterruptStackFrame,
    _error_code: u64,
) {
}

extern "x86-interrupt" fn double_fault_handler(
    frame: x86_64::structures::idt::InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", frame);
}

extern "x86-interrupt" fn page_fault_handler(
    _frame: x86_64::structures::idt::InterruptStackFrame,
    _error_code: x86_64::structures::idt::PageFaultErrorCode,
) {
}

/// Returns `true` during processing of an external interrupt, and `false` at
/// all other times.
pub fn is_external_handler_context() -> bool {
    REGISTRY.peek().is_external_context()
}
