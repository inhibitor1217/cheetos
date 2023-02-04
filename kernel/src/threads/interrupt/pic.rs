use super::mutex::Mutex;

struct Pic {
    offset: u8,
    control: x86_64::instructions::port::Port<u8>,
    data: x86_64::instructions::port::Port<u8>,
}

impl Pic {
    fn can_handle(&self, interrupt_id: u8) -> bool {
        (self.offset..self.offset + 8).contains(&interrupt_id)
    }
}

/// 8259A Programmable Interrupt Controller.
///
/// By default, interrupts 0...15 delivered by the PICs will go to interrupt
/// vectors 0...15. Those vectors are also used for CPU traps and exceptions,
/// so we reprogram the PICs so that interrupts 0...15 are delivered to
/// interrupt vectors `offset`...`offset + 15` instead.
pub struct Pics(Pic, Pic);

impl Pics {
    /// Programmable Interrupt Controller registers.
    /// A PC has two PICs, called the master and slave PICs, with the slave
    /// attached ("cascaded") to the master IRQ line 2.

    /// Master PIC control register address.
    const PIC0_CONTROL: u16 = 0x20;

    /// Master PIC data register address.
    const PIC0_DATA: u16 = 0x21;

    /// Slave PIC control register address.
    const PIC1_CONTROL: u16 = 0xa0;

    /// Slave PIC data register address.
    const PIC1_DATA: u16 = 0xa1;

    /// Creates a new pair of PICs with given register offsets.
    pub const fn new(offset: u8) -> Pics {
        Pics(
            Pic {
                offset,
                control: x86_64::instructions::port::Port::new(Self::PIC0_CONTROL),
                data: x86_64::instructions::port::Port::new(Self::PIC0_DATA),
            },
            Pic {
                offset: offset + 8,
                control: x86_64::instructions::port::Port::new(Self::PIC1_CONTROL),
                data: x86_64::instructions::port::Port::new(Self::PIC1_DATA),
            },
        )
    }

    /// Initialize the PICs.
    ///
    /// # Safety
    /// This function in unsafe because the caller must ensure that the PICs
    /// are created with the correct offests. Also, this function should be
    /// called only once.
    pub unsafe fn init(&mut self) {
        let Self(master, slave) = self;

        // Mask all interrupts on both PICs.
        master.data.write(0xff);
        slave.data.write(0xff);

        // Initialize master.
        master.control.write(0x11); // ICW1: single mode, edge triggered, expect ICW4.
        master.data.write(master.offset); // ICW2: line IR0...7 -> irq offset...offset + 7.
        master.data.write(0x04); // ICW3: slave PIC on line IR2.
        master.data.write(0x01); // ICW4: 8086 mode, normal EOI, non-buffered.

        // Initialize slave.
        slave.control.write(0x11); // ICW1: single mode, edge triggered, expect ICW4.
        slave.data.write(slave.offset); // ICW2: line IR0..IR7 -> irq offset...offset + 7.
        slave.data.write(0x02); // ICW3: slave ID is 2.
        slave.data.write(0x01); // ICW4: 8086 mode, normal EOI, non-buffered.

        // Unmask all interrupts.
        master.data.write(0x00);
        slave.data.write(0x00);
    }

    /// Sends an end-of-interrupt signal to the PIC for the given
    /// `interrupt_id`.
    ///
    /// If we don't acknowledge the IRQ, it will never be delivered to us again,
    /// so it is important.
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure that this
    /// function is called at the tail of the external interrupt handler.
    pub unsafe fn end_of_interrupt(&mut self, interrupt_id: u8) {
        let Self(master, slave) = self;

        assert!(master.can_handle(interrupt_id) || slave.can_handle(interrupt_id));

        // Acknowledge master PIC.
        master.control.write(0x20);

        // Acknowledge slave PIC if this is a slave interrupt.
        if slave.can_handle(interrupt_id) {
            slave.control.write(0x20);
        }
    }
}

/// Global PIC registers.
pub static PICS: Mutex<Pics> = Mutex::new(Pics::new(0x20));
