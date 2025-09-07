use std::cell::RefCell;
use std::rc::Rc;
use crate::cpu::registers::Registers;
use crate::mmu::Mmu;

use anyhow::Result;
use crate::cpu::instruction_set::{DebugInstruction, InstructionSet, Operation};
use crate::cpu::interrupts::Interrupts;
use crate::EmulationMode;

mod registers;
mod interrupts;
pub mod instruction_set;

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

    pub fn emulation_loop(&mut self, mode: EmulationMode) -> Result<()> {
        match mode {
            EmulationMode::Debug(iterations) => {
                Ok(for i in 0..iterations {
                    self.run_next_opcode()?
                })
            }
            EmulationMode::Normal => {
                loop {
                    self.run_next_opcode()?
                }
            }
        }
    }

    pub fn run_next_opcode(&mut self) -> Result<()> {
        let opcode = self.read_at_program_counter()?;

        let instruction = self.instruction_set.fetch_instruction(opcode);

        match instruction.operation {
            Operation::None => {
                if instruction.name == "" {
                    panic!("Unimplemented opcode {:#x}", opcode);
                }

                //println!("PC: 0x{:X}, SP: 0x{:X}, Z: {} -> NOP", self.registers.pc() - 1, self.registers.sp(), self.registers.zero_flag());
            }
            Operation::Nullary(ref operation) => {
                assert_eq!(instruction.length, 1);

                //println!("PC: 0x{:X}, SP: 0x{:X}, Z: {} -> {}", self.registers.pc() - 1, self.registers.sp(), self.registers.zero_flag(), instruction);

                operation(&mut self.mmu.borrow_mut(), &mut self.registers);
            }
            Operation::Unary(ref operation) => {
                assert_eq!(instruction.length, 2);

                let operand = self.read_at_program_counter()?;

                //println!("PC: 0x{:X}, SP: 0x{:X}, Z: {} -> {} ({:02X})", self.registers.pc() - 2, self.registers.sp(), self.registers.zero_flag(), instruction, operand);

                operation(&mut self.mmu.borrow_mut(), &mut self.registers, operand);
            }
            Operation::Binary(ref operation) => {
                assert_eq!(instruction.length, 3);

                let first_operand = self.read_at_program_counter()?;
                let second_operand = self.read_at_program_counter()?;

                //println!("PC: 0x{:X}, SP: 0x{:X}, Z: {} -> {} (0x{:02X} 0x{:02X})", self.registers.pc() - 3, self.registers.sp(), self.registers.zero_flag(), instruction, first_operand, second_operand);

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

    // Fetches a range of instructions, backwards and forwards from the pc
    // Used for debugging
    pub fn get_instruction_range_from_address(&self, address: usize, backward_count: u32, forward_count: u32) -> Result<Vec<DebugInstruction>> {
        let mut pc = address;
        let mut instructions = vec![];

        for i in 0..=forward_count {
            let mmu = self.mmu.borrow();

            let address = pc;

            let opcode = mmu.read_byte(pc)?;
            let instruction = self.instruction_set.fetch_instruction(opcode);

            pc += 1;

            let (first_operand, second_operand) = match instruction.length {
                1 => (None, None),
                2 => {
                    let operand = mmu.read_byte(pc).unwrap();
                    pc += 1;

                    (Some(operand), None)
                },
                3 => {
                    let first_operand = mmu.read_byte(pc).unwrap();
                    pc += 1;

                    let second_operand = mmu.read_byte(pc).unwrap();
                    pc += 1;

                    (Some(first_operand), Some(second_operand))
                },
                _ => (None, None),
            };

            let name = if instruction.name == "" { "Unimplemented instruction " } else { instruction.name };

            instructions.push(
            DebugInstruction {
                    address,
                    opcode,
                    first_operand,
                    second_operand,
                    name: name.to_string(),
                }
            )
        }

        Ok(instructions)
    }
}