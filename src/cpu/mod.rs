use std::cell::RefCell;
use std::rc::Rc;
use crate::cpu::registers::Registers;
use crate::mmu::Mmu;

use anyhow::Result;
use crate::cpu::interrupts::Interrupts;

mod registers;
mod interrupts;

pub struct Cpu {
    mmu: Rc<RefCell<Mmu>>,
    pub registers: Registers,
    interrupts: Interrupts,
}

impl Cpu {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        Cpu {
            mmu: mmu.clone(),
            registers: Registers::new(),
            interrupts: Interrupts::new(mmu.clone()),
        }
    }

    pub fn emulation_loop(&self) {

    }

    pub fn run_next_opcode(&self) -> Result<()> {
        let opcode = self.mmu.borrow().read_byte(self.registers.pc() as usize)?;
        todo!()
    }
}