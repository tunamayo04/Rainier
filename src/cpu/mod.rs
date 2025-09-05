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
        loop {
            self.run_next_opcode()?;
        }
    }

    pub fn run_next_opcode(&mut self) -> Result<()> {
        let opcode = self.read_at_program_counter()?;

        let instruction = self.instruction_set.fetch_instruction(opcode);

        println!("PC: {:#X}, SP: {:#X} -> {}", self.registers.pc() - 1, self.registers.sp(), instruction);

        match instruction.operation {
            Operation::None => {
                if instruction.name == "" {
                    panic!("Unimplemented opcode {:#x}", opcode);
                }
            }
            Operation::Nullary(operation) => {
                assert_eq!(instruction.length, 1);

                operation(&mut self.mmu.borrow_mut(), &mut self.registers);
            }
            Operation::Unary(operation) => {
                assert_eq!(instruction.length, 2);

                let operand = self.read_at_program_counter()?;

                operation(&mut self.mmu.borrow_mut(), &mut self.registers, operand);
            }
            Operation::Binary(operation) => {
                assert_eq!(instruction.length, 3);

                let first_operand = self.read_at_program_counter()?;
                let second_operand = self.read_at_program_counter()?;

                operation(&mut self.mmu.borrow_mut(), &mut self.registers, first_operand, second_operand);
            }
        }

        Ok(())
    }

    // Reads the value in memory pointed at by PC and increments PC
    fn read_at_program_counter(&mut self) -> Result<u8> {
        let value = self.mmu.borrow().read_byte(self.registers.pc() as usize)?;
        self.registers.increment_pc();

        Ok(value)
    }
}