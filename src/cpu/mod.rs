use std::cell::RefCell;
use std::rc::Rc;
use crate::cpu::registers::Registers;
use crate::mmu::Mmu;

use anyhow::Result;
use crate::cpu::instruction_set::{InstructionSet, Operation};
use crate::cpu::interrupts::Interrupts;

mod registers;
mod interrupts;
mod instruction_set;

pub struct Cpu {
    mmu: Rc<RefCell<Mmu>>,
    pub registers: Registers,
    interrupts: Interrupts,
    instruction_set: InstructionSet,
}

impl Cpu {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        Cpu {
            mmu: mmu.clone(),
            registers: Registers::new(),
            interrupts: Interrupts::new(mmu.clone()),
            instruction_set: InstructionSet::new(mmu.clone()),
        }
    }

    pub fn emulation_loop(&mut self) -> Result<()> {
        self.run_next_opcode()?;
        self.run_next_opcode()
    }

    pub fn run_next_opcode(&mut self) -> Result<()> {
        let opcode = self.mmu.borrow().read_byte(self.registers.pc() as usize)?;
        let instruction = self.instruction_set.fetch_instruction(opcode);

        self.registers.increment_pc();

        println!("PC: {:#x} -> {}",self.registers.pc() ,instruction);

        match &instruction.operation {
            Operation::None => {
                if instruction.name == "" {
                    panic!("Unimplemented opcode {:#x}", opcode);
                }
            }
            Operation::Nullary(op) => {
                assert_eq!(instruction.length, 1);

                op(&mut self.mmu.borrow_mut(), &mut self.registers);
            }
            Operation::Unary(_) => {}
            Operation::Binary(_) => {}
        }

        Ok(())
    }
}