use std::cell::{RefCell};
use std::{fmt, mem};
use std::mem::MaybeUninit;
use std::rc::Rc;
use crate::cpu::registers::{Register, Registers};
use crate::mmu::Mmu;

type NullaryOperation = Box<dyn Fn(&mut Mmu, &mut Registers)>;
type UnaryOperation = Box<dyn Fn(&mut Mmu, u8)>;
type BinaryOperation = Box<dyn Fn(&mut Mmu, u8, u8)>;

#[derive(Default)]
pub enum Operation {
    #[default]
    None,
    Nullary(NullaryOperation),
    Unary(UnaryOperation),
    Binary(BinaryOperation),
}

#[derive(Default)]
pub struct Instruction {
    pub name: &'static str,
    pub opcode: u8,
    pub length: usize,
    pub cycles: usize,
    pub operation: Operation,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (opcode: 0x{:02X}, length: {}, cycles: {})",
            self.name, self.opcode, self.length, self.cycles
        )
    }
}

pub struct InstructionSet {
    instructions: [Instruction; 256],

    mmu: Rc<RefCell<Mmu>>,
}

impl InstructionSet {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> InstructionSet {
        let mut instructions: [MaybeUninit<Instruction>; 256] = unsafe {MaybeUninit::uninit().assume_init() };

        for i in 0..256 {
            instructions[i] = MaybeUninit::new(Instruction::default());
        }

        let mut instructions: [Instruction; 256] = unsafe { mem::transmute(instructions) };

        instructions[0x00] = Instruction{ name: "NOP", opcode: 0x00, length: 1, cycles: 1, operation: Operation::None, };
        instructions[0x03] = Instruction{ name: "INC BC", opcode: 0x03, length: 1, cycles: 2,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::BC ) })) };
        instructions[0x04] = Instruction{ name: "INC B", opcode: 0x04, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::B) })) };
        instructions[0x05] = Instruction{ name: "DEC B", opcode: 0x05, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::B) })) };
        instructions[0x0C] = Instruction{ name: "INC C", opcode: 0x0C, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::C) })) };
        instructions[0x0D] = Instruction{ name: "DEC C", opcode: 0x0D, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::C) })) };

        instructions[0x13] = Instruction{ name: "INC DE", opcode: 0x13, length: 1, cycles: 2,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::DE ) })) };
        instructions[0x14] = Instruction{ name: "INC D", opcode: 0x14, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::D) })) };
        instructions[0x15] = Instruction{ name: "DEC D", opcode: 0x15, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::D) })) };
        instructions[0x1C] = Instruction{ name: "INC E", opcode: 0x1C, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::E) })) };
        instructions[0x1D] = Instruction{ name: "DEC E", opcode: 0x1D, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::E) })) };

        instructions[0x23] = Instruction{ name: "INC HL", opcode: 0x23, length: 1, cycles: 2,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::HL ) })) };
        instructions[0x24] = Instruction{ name: "INC H", opcode: 0x24, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::H) })) };
        instructions[0x25] = Instruction{ name: "DEC H", opcode: 0x25, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::H) })) };
        instructions[0x2C] = Instruction{ name: "INC L", opcode: 0x2C, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::L) })) };
        instructions[0x2D] = Instruction{ name: "DEC L", opcode: 0x2D, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::L) })) };

        instructions[0x33] = Instruction{ name: "INC SP", opcode: 0x33, length: 1, cycles: 2,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::SP ) })) };
        instructions[0x3C] = Instruction{ name: "INC A", opcode: 0x3C, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::A) })) };
        instructions[0x3D] = Instruction{ name: "DEC A", opcode: 0x3D, length: 1, cycles: 1,
            operation: Operation::Nullary(Box::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::A) })) };

        InstructionSet { instructions, mmu }
    }

    pub fn fetch_instruction(&self, opcode: u8) -> &Instruction {
        &self.instructions[opcode as usize]
    }

    // Increment the contents of a register by 1
    // Flags: Z 0 8-bit -
    pub fn inc_8bit(registers: &mut Registers, register: Register) {
        let original_value = registers.get_8bit_register(register.clone());
        let new_value = original_value + 1;
        registers.set_8bit_register(register, new_value);

        if new_value == 0 {
            registers.set_zero_flag(true);
        }

        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(Self::carry_check_add_8bit(original_value, 1));
    }

    // Decrement the contents of a register by 1
    // Flags: Z 1 8-bit -
    pub fn dec_8bit(registers: &mut Registers, register: Register) {
        let original_value = registers.get_8bit_register(register.clone());
        let new_value = original_value - 1;
        registers.set_8bit_register(register, new_value);

        if new_value == 0 {
            registers.set_zero_flag(true);
        }

        registers.set_subtraction_flag(true);
        registers.set_half_carry_flag(Self::carry_check_sub_8bit(original_value, 1));
    }

    // Increment the contents of a register pair by 1
    // Flags: - - - -
    pub fn inc_16bit(registers: &mut Registers, register: Register) {
        let original_value = registers.get_16bit_register(register.clone());
        let new_value = original_value + 1;
        registers.set_16bit_register(register, new_value);
    }

    // Decrements the contents of a register pair by 1
    // Flags: - - - -
    pub fn dec_16bit(registers: &mut Registers, register: Register) {
        let original_value = registers.get_16bit_register(register.clone());
        let new_value = original_value - 1;
        registers.set_16bit_register(register, new_value);
    }

    fn carry_check_add_8bit(a: u8, b: u8) -> bool {
        (((a & 0xF) + (b & 0xF)) & 0x10) == 0x10
    }

    fn carry_check_sub_8bit(a: u8, b: u8) -> bool {
        (((a & 0xF) - (b & 0xF)) & 0x10) == 0x10
    }
}