use crate::cpu::registers::Registers;

mod registers;

pub struct Cpu {
    registers: Registers,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::new(),
        }
    }
}