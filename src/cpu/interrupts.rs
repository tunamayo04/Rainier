use std::cell::RefCell;
use std::rc::Rc;
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
    registers: Registers
}

impl Interrupts {
    pub fn new(mmu: Rc<RefCell<Mmu>>, registers: Registers) -> Self {
        Interrupts { mmu, registers }
    }

    pub fn get_interrupt_master_enable(&self) -> bool {
        self.mmu.borrow().ime() == 1
    }

    pub fn get_interrupt_enable_register(&self) -> u8 {
        self.mmu.borrow().ie()
    }

    pub fn get_interrupt_flag_register(&self) -> u8 {
        self.mmu.borrow().iflag()
    }

    pub fn handle_interrupts(&mut self) -> bool {
        // No interrupt requested
        if self.get_interrupt_enable_register() == 0 || self.get_interrupt_flag_register() == 0 { return false; }

        for interrupt in Interrupt::VALUES {
            if self.get_interrupt_enable_register() & (1 << interrupt as u8) != 0 && self.get_interrupt_flag_register() & (1 << interrupt as u8) != 0 {
                if self.get_interrupt_master_enable() {
                    match interrupt {
                        Interrupt::VBlank => self.registers.set_pc(0x40),
                        Interrupt::LCD => self.registers.set_pc(0x48),
                        Interrupt::Timer => self.registers.set_pc(0x50),
                        Interrupt::Serial => self.registers.set_pc(0x58),
                        Interrupt::Joypad => self.registers.set_pc(0x60),
                    }
                }
                return true;
            }
        }

        return false;
    }
}