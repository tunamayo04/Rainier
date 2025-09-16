use std::cell::RefCell;
use std::rc::Rc;
use crate::cpu::registers::Registers;
use crate::mmu::{MemoryRegion, Mmu};

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
        let registers = Registers::new();
        Cpu {
            mmu: mmu.clone(),
            registers: registers.clone(),
            interrupts: Interrupts::new(mmu.clone(), registers.clone()),
            instruction_set: InstructionSet::new(mmu.clone()),
        }
    }

    pub fn emulation_loop(&mut self, mode: EmulationMode) -> Result<()> {
        match mode {
            EmulationMode::Debug(iterations) => {
                Ok(for i in 0..iterations {
                    self.run_next_opcode()?;
                    self.interrupts.handle_interrupts();
                })
            }
            EmulationMode::Normal => {
                loop {
                    self.run_next_opcode()?;
                    self.interrupts.handle_interrupts();
                }
            }
        }
    }

    pub fn run_next_opcode(&mut self) -> Result<()> {
        let mut opcode = self.read_at_program_counter()?;
        let mut instruction = self.instruction_set.fetch_instruction(opcode);


        // 16-bit opcodes
        let is_16bit_opcode = if opcode == 0xCB {
            opcode = self.read_at_program_counter()?;
            instruction = self.instruction_set.fetch_instruction_16bit(opcode);

            true
        } else {
           false
        };

        match instruction.operation {
            Operation::None => {
                if instruction.name == "" {
                    if is_16bit_opcode {
                        panic!("Unimplemented opcode 0xCB{:X} at {:#X}", opcode, self.registers.pc() - 1);
                    }
                    else {
                        panic!("Unimplemented opcode {:#X} at {:#X}", opcode, self.registers.pc());
                    }
                }
            }
            Operation::Nullary(ref operation) => {
                operation(&mut self.mmu.borrow_mut(), &mut self.registers);
            }
            Operation::Unary(ref operation) => {
                let operand = self.read_at_program_counter()?;

                operation(&mut self.mmu.borrow_mut(), &mut self.registers, operand);
            }
            Operation::Binary(ref operation) => {
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

    // Get all the instructions in the ROM and the id of the instruction corresponding to a given PC
    pub fn dump_instructions(&self, current_address: usize) -> Vec<DebugInstruction> {
        let mut pc = 0;
        let mut instructions = vec![];

        let mmu = self.mmu.borrow();
        let memory = mmu.dump_memory_region(MemoryRegion::RomBankZero);

        while pc < memory.len() {
            // Cartridge header
            if pc >= 0x104 && pc < 0x150 {
                pc = 0x150;
                continue;
            }

            let address = pc;

            let opcode = *memory.get(address).unwrap();
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

            let name = if instruction.name == "" { String::from("Unimplemented instruction ") } else { instruction.name };

            instructions.push(
                DebugInstruction {
                    address,
                    opcode,
                    first_operand,
                    second_operand,
                    name,
                }
            )
        }

        instructions
    }
}