use std::cell::RefCell;
use std::rc::Rc;
use crate::bit_utils::split_2bytes;
use crate::cpu::registers::Registers;
use crate::mmu::Mmu;

#[derive(Copy, Clone)]
pub enum Interrupt {
    VBlank = 0,
    LCD = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}

impl Interrupt {
    pub const VALUES: [Self; 5] = [Self::VBlank, Self::LCD, Self::Timer, Self::Serial, Self::Joypad];
}

pub struct Interrupts {
    mmu: Rc<RefCell<Mmu>>,
}

impl Interrupts {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        Interrupts { mmu }
    }

    pub fn get_interrupt_enable_register(&self) -> u8 {
        self.mmu.borrow().ie()
    }

    pub fn get_interrupt_flag_register(&self) -> u8 {
        self.mmu.borrow().iflag()
    }
    pub fn set_interrupt_flag_registers(&mut self, value: u8) { self.mmu.borrow_mut().set_iflag(value); }

    pub fn handle_interrupts(&mut self, registers: &mut Registers) -> bool {
        // No interrupt requested
        if self.get_interrupt_enable_register() == 0 || self.get_interrupt_flag_register() == 0 { return false; }

        for interrupt in Interrupt::VALUES {
            if self.get_interrupt_enable_register() & (1 << interrupt as u8) != 0 && self.get_interrupt_flag_register() & (1 << interrupt as u8) != 0 {
                if registers.ime() {
                    // Reset interrupt
                    registers.set_ime(false);
                    let mut IF = self.get_interrupt_flag_register();
                    IF &= !(1 << interrupt as u8);
                    self.set_interrupt_flag_registers(IF);

                    // Load current PC in stack
                    let mut mmu = self.mmu.borrow_mut();
                    let (return_lower, return_higher) = split_2bytes(registers.pc());

                    registers.decrement_sp();
                    mmu.write_byte(registers.sp() as usize, return_higher).unwrap();
                    registers.decrement_sp();
                    mmu.write_byte(registers.sp() as usize, return_lower).unwrap();

                    match interrupt {
                        Interrupt::VBlank => registers.set_pc(0x40),
                        Interrupt::LCD => registers.set_pc(0x48),
                        Interrupt::Timer => registers.set_pc(0x50),
                        Interrupt::Serial => registers.set_pc(0x58),
                        Interrupt::Joypad => registers.set_pc(0x60),
                    }
                }
                return true;
            }
        }

        false
    }
}