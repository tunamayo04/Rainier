use std::cell::{RefCell};
use std::{fmt, mem};
use std::mem::MaybeUninit;
use std::rc::Rc;
use crate::bit_utils::{carry_check_add_8bit, carry_check_sub_8bit, concatenate_bytes, split_2bytes};
use crate::cpu::registers::{Register, Registers};
use crate::mmu::Mmu;

type NullaryOperation = Rc<dyn Fn(&mut Mmu, &mut Registers)>;
type UnaryOperation = Rc<dyn Fn(&mut Mmu, &mut Registers, u8)>;
type BinaryOperation = Rc<dyn Fn(&mut Mmu, &mut Registers, u8, u8)>;

#[derive(Default, Clone)]
pub enum Operation {
    #[default]
    None,
    Nullary(NullaryOperation),
    Unary(UnaryOperation),
    Binary(BinaryOperation),
}

#[derive(Default, Clone)]
pub struct Instruction {
    pub name: &'static str,
    pub opcode: u8,
    pub length: usize,
    pub cycles: usize,
    pub operation: Operation,
}

pub struct DebugInstruction {
    pub address: usize,
    pub opcode: u8,
    pub first_operand: Option<u8>,
    pub second_operand: Option<u8>,
    pub name: String,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02X} {}", self.opcode, self.name)
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
        instructions[0x01] = Instruction{ name: "LD BC, d16", opcode: 0x01, length: 3, cycles: 3,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_byte: u8, higher_byte: u8| { Self::ld_16bit(registers, Register::BC, lower_byte, higher_byte) })) };
        instructions[0x02] = Instruction{ name: "LD (BC), A", opcode: 0x02, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit_mem(mmu, registers.bc(), registers.a()) })) } ;
        instructions[0x03] = Instruction{ name: "INC BC", opcode: 0x03, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::BC ) })) };
        instructions[0x04] = Instruction{ name: "INC B", opcode: 0x04, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::B) })) };
        instructions[0x05] = Instruction{ name: "DEC B", opcode: 0x05, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::B) })) };
        instructions[0x06] = Instruction{ name: "LD B, d8", opcode: 0x06, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::B, value) })) } ;
        instructions[0x0A] = Instruction{ name: "LD A, (BC)", opcode: 0x0A, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit(registers, Register::A, mmu.read_byte(registers.bc() as usize).unwrap()) })) };
        instructions[0x0B] = Instruction{ name: "DEC BC", opcode: 0x0B, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::dec_16bit(registers, Register::BC) })) };
        instructions[0x0C] = Instruction{ name: "INC C", opcode: 0x0C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::C) })) };
        instructions[0x0D] = Instruction{ name: "DEC C", opcode: 0x0D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::C) })) };
        instructions[0x0E] = Instruction{ name: "LD C, d8", opcode: 0x0E, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::C, value) })) } ;

        instructions[0x11] = Instruction{ name: "LD DE, d16", opcode: 0x11, length: 3, cycles: 3,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_byte: u8, higher_byte: u8| { Self::ld_16bit(registers, Register::DE, lower_byte, higher_byte) })) };
        instructions[0x12] = Instruction{ name: "LD (DE), A", opcode: 0x12, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit_mem(mmu, registers.de(), registers.a()) })) } ;
        instructions[0x13] = Instruction{ name: "INC DE", opcode: 0x13, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::DE ) })) };
        instructions[0x14] = Instruction{ name: "INC D", opcode: 0x14, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::D) })) };
        instructions[0x15] = Instruction{ name: "DEC D", opcode: 0x15, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::D) })) };
        instructions[0x16] = Instruction{ name: "LD D, d8", opcode: 0x16, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::D, value) })) } ;
        instructions[0x1A] = Instruction{ name: "LD A, (DE)", opcode: 0x1A, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit(registers, Register::A, mmu.read_byte(registers.de() as usize).unwrap()) })) };
        instructions[0x1B] = Instruction{ name: "DEC DE", opcode: 0x1B, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::dec_16bit(registers, Register::DE) })) };
        instructions[0x1C] = Instruction{ name: "INC E", opcode: 0x1C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::E) })) };
        instructions[0x1D] = Instruction{ name: "DEC E", opcode: 0x1D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::E) })) };
        instructions[0x1E] = Instruction{ name: "LD E, d8", opcode: 0x1E, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::E, value) })) } ;

        instructions[0x20] = Instruction{ name: "JR NZ, s8", opcode: 0x20, length: 2, cycles: 3, // TODO: Check the variable cycles implementation
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, steps: u8| { if registers.zero_flag() == false { Self::jr(registers, steps) } })) };
        instructions[0x21] = Instruction{ name: "LD HL, d16", opcode: 0x21, length: 3, cycles: 3,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_byte: u8, higher_byte: u8| { Self::ld_16bit(registers, Register::HL, lower_byte, higher_byte) })) };
        instructions[0x22] = Instruction{ name: "LD (HL+), A", opcode: 0x22, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| {
                Self::ld_8bit_mem(mmu, registers.hl(), registers.a());
                registers.set_hl(registers.hl() + 1) })) } ;
        instructions[0x23] = Instruction{ name: "INC HL", opcode: 0x23, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::HL ) })) };
        instructions[0x24] = Instruction{ name: "INC H", opcode: 0x24, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::H) })) };
        instructions[0x25] = Instruction{ name: "DEC H", opcode: 0x25, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::H) })) };
        instructions[0x26] = Instruction{ name: "LD H, d8", opcode: 0x26, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::H, value) })) } ;
        instructions[0x2A] = Instruction{ name: "LD A, (HL+)", opcode: 0x2A, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| {
                Self::ld_8bit(registers, Register::A, mmu.read_byte(registers.hl() as usize).unwrap());
                registers.set_hl(registers.hl() + 1)})) };
        instructions[0x3B] = Instruction{ name: "DEC HL", opcode: 0x3B, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::dec_16bit(registers, Register::HL) })) };
        instructions[0x2C] = Instruction{ name: "INC L", opcode: 0x2C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::L) })) };
        instructions[0x2D] = Instruction{ name: "DEC L", opcode: 0x2D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::L) })) };
        instructions[0x2E] = Instruction{ name: "LD L, d8", opcode: 0x2E, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::L, value) })) } ;

        instructions[0x30] = Instruction{ name: "JR NC, s8", opcode: 0x30, length: 2, cycles: 3, // TODO: Check the variable cycles implementation
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, steps: u8| { if registers.carry_flag() == false { Self::jr(registers, steps) } })) };
        instructions[0x31] = Instruction{ name: "LD SP, d16", opcode: 0x31, length: 3, cycles: 3,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_byte: u8, higher_byte: u8| { Self::ld_16bit(registers, Register::SP, lower_byte, higher_byte) })) };
        instructions[0x32] = Instruction{ name: "LD (HL-), A", opcode: 0x32, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| {
                Self::ld_8bit_mem(mmu, registers.hl(), registers.a());
                registers.set_hl(registers.hl() - 1) })) } ;
        instructions[0x33] = Instruction{ name: "INC SP", opcode: 0x33, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::SP ) })) };
        instructions[0x36] = Instruction{ name: "LD (HL), d8", opcode: 0x36, length: 2, cycles: 3,
            operation: Operation::Unary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, value: u8| { Self::ld_8bit_mem(mmu, registers.hl(), value) })) } ;
        instructions[0x3A] = Instruction{ name: "LD A, (HL-)", opcode: 0x3A, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| {
                Self::ld_8bit(registers, Register::A, mmu.read_byte(registers.hl() as usize).unwrap());
                registers.set_hl(registers.hl() - 1)})) };
        instructions[0x3B] = Instruction{ name: "DEC SP", opcode: 0x3B, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::dec_16bit(registers, Register::SP) })) };
        instructions[0x3C] = Instruction{ name: "INC A", opcode: 0x3C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::A) })) };
        instructions[0x3D] = Instruction{ name: "DEC A", opcode: 0x3D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::A) })) };
        instructions[0x3E] = Instruction{ name: "LD A, d8", opcode: 0x3E, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::A, value) })) } ;

        instructions[0x40] = Instruction{ name: "LD B, B", opcode: 0x40, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::B, registers.b()) })) };
        instructions[0x41] = Instruction{ name: "LD B, C", opcode: 0x41, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::B, registers.c()) })) };
        instructions[0x42] = Instruction{ name: "LD B, D", opcode: 0x42, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::B, registers.d()) })) };
        instructions[0x43] = Instruction{ name: "LD B, E", opcode: 0x43, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::B, registers.e()) })) };
        instructions[0x44] = Instruction{ name: "LD B, H", opcode: 0x44, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::B, registers.h()) })) };
        instructions[0x45] = Instruction{ name: "LD B, L", opcode: 0x45, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::B, registers.l()) })) };
        instructions[0x46] = Instruction{ name: "LD B, (HL)", opcode: 0x46, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit(registers, Register::B, mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x47] = Instruction{ name: "LD B, A", opcode: 0x47, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::B, registers.a()) })) };
        instructions[0x48] = Instruction{ name: "LD C, B", opcode: 0x48, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::C, registers.b()) })) };
        instructions[0x49] = Instruction{ name: "LD C, C", opcode: 0x49, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::C, registers.c()) })) };
        instructions[0x4A] = Instruction{ name: "LD C, D", opcode: 0x4A, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::C, registers.d()) })) };
        instructions[0x4B] = Instruction{ name: "LD C, E", opcode: 0x4B, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::C, registers.e()) })) };
        instructions[0x4C] = Instruction{ name: "LD C, H", opcode: 0x4C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::C, registers.h()) })) };
        instructions[0x4D] = Instruction{ name: "LD C, L", opcode: 0x4D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::C, registers.l()) })) };
        instructions[0x4E] = Instruction{ name: "LD C, (HL)", opcode: 0x4E, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit(registers, Register::C, mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x4F] = Instruction{ name: "LD C, A", opcode: 0x4F, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::C, registers.a()) })) };

        instructions[0x50] = Instruction{ name: "LD D, B", opcode: 0x50, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::D, registers.b()) })) };
        instructions[0x51] = Instruction{ name: "LD D, C", opcode: 0x51, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::D, registers.c()) })) };
        instructions[0x52] = Instruction{ name: "LD D, D", opcode: 0x52, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::D, registers.d()) })) };
        instructions[0x53] = Instruction{ name: "LD D, E", opcode: 0x53, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::D, registers.e()) })) };
        instructions[0x54] = Instruction{ name: "LD D, H", opcode: 0x54, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::D, registers.h()) })) };
        instructions[0x55] = Instruction{ name: "LD D, L", opcode: 0x55, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::D, registers.l()) })) };
        instructions[0x56] = Instruction{ name: "LD D, (HL)", opcode: 0x56, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit(registers, Register::D, mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x57] = Instruction{ name: "LD D, A", opcode: 0x57, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::D, registers.a()) })) };
        instructions[0x58] = Instruction{ name: "LD E, B", opcode: 0x58, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::E, registers.b()) })) };
        instructions[0x59] = Instruction{ name: "LD E, C", opcode: 0x59, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::E, registers.c()) })) };
        instructions[0x5A] = Instruction{ name: "LD E, D", opcode: 0x5A, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::E, registers.d()) })) };
        instructions[0x5B] = Instruction{ name: "LD E, E", opcode: 0x5B, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::E, registers.e()) })) };
        instructions[0x5C] = Instruction{ name: "LD E, H", opcode: 0x5C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::E, registers.h()) })) };
        instructions[0x5D] = Instruction{ name: "LD E, L", opcode: 0x5D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::E, registers.l()) })) };
        instructions[0x5E] = Instruction{ name: "LD E, (HL)", opcode: 0x5E, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit(registers, Register::E, mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x5F] = Instruction{ name: "LD E, A", opcode: 0x5F, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::E, registers.a()) })) };

        instructions[0x60] = Instruction{ name: "LD H, B", opcode: 0x60, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::H, registers.b()) })) };
        instructions[0x61] = Instruction{ name: "LD H, C", opcode: 0x61, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::H, registers.c()) })) };
        instructions[0x62] = Instruction{ name: "LD H, D", opcode: 0x62, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::H, registers.d()) })) };
        instructions[0x63] = Instruction{ name: "LD H, E", opcode: 0x63, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::H, registers.e()) })) };
        instructions[0x64] = Instruction{ name: "LD H, H", opcode: 0x64, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::H, registers.h()) })) };
        instructions[0x65] = Instruction{ name: "LD H, L", opcode: 0x65, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::H, registers.l()) })) };
        instructions[0x66] = Instruction{ name: "LD H, (HL)", opcode: 0x66, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit(registers, Register::H, mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x67] = Instruction{ name: "LD H, A", opcode: 0x67, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::H, registers.a()) })) };
        instructions[0x68] = Instruction{ name: "LD L, B", opcode: 0x68, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::L, registers.b()) })) };
        instructions[0x69] = Instruction{ name: "LD L, C", opcode: 0x69, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::L, registers.c()) })) };
        instructions[0x6A] = Instruction{ name: "LD L, D", opcode: 0x6A, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::L, registers.d()) })) };
        instructions[0x6B] = Instruction{ name: "LD L, E", opcode: 0x6B, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::L, registers.e()) })) };
        instructions[0x6C] = Instruction{ name: "LD L, H", opcode: 0x6C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::L, registers.h()) })) };
        instructions[0x6D] = Instruction{ name: "LD L, L", opcode: 0x6D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::L, registers.l()) })) };
        instructions[0x6E] = Instruction{ name: "LD L, (HL)", opcode: 0x6E, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit(registers, Register::L, mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x6F] = Instruction{ name: "LD L, A", opcode: 0x6F, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::L, registers.a()) })) };

        instructions[0x70] = Instruction{ name: "LD (HL), B", opcode: 0x70, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit_mem(mmu, registers.hl(), registers.b()) })) } ;
        instructions[0x71] = Instruction{ name: "LD (HL), C", opcode: 0x71, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit_mem(mmu, registers.hl(), registers.c()) })) } ;
        instructions[0x72] = Instruction{ name: "LD (HL), D", opcode: 0x71, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit_mem(mmu, registers.hl(), registers.d()) })) } ;
        instructions[0x73] = Instruction{ name: "LD (HL), E", opcode: 0x73, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit_mem(mmu, registers.hl(), registers.e()) })) } ;
        instructions[0x74] = Instruction{ name: "LD (HL), H", opcode: 0x74, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit_mem(mmu, registers.hl(), registers.h()) })) } ;
        instructions[0x75] = Instruction{ name: "LD (HL), L", opcode: 0x75, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit_mem(mmu, registers.hl(), registers.l()) })) } ;
        instructions[0x77] = Instruction{ name: "LD (HL), A", opcode: 0x77, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit_mem(mmu, registers.hl(), registers.a()) })) } ;
        instructions[0x78] = Instruction{ name: "LD A, B", opcode: 0x78, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::A, registers.b()) })) };
        instructions[0x79] = Instruction{ name: "LD A, C", opcode: 0x79, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::A, registers.c()) })) };
        instructions[0x7A] = Instruction{ name: "LD A, D", opcode: 0x7A, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::A, registers.d()) })) };
        instructions[0x7B] = Instruction{ name: "LD A, E", opcode: 0x7B, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::A, registers.e()) })) };
        instructions[0x7C] = Instruction{ name: "LD A, H", opcode: 0x7C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::A, registers.h()) })) };
        instructions[0x7D] = Instruction{ name: "LD A, L", opcode: 0x7D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::A, registers.l()) })) };
        instructions[0x7E] = Instruction{ name: "LD A, (HL)", opcode: 0x7E, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit(registers, Register::A, mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x7F] = Instruction{ name: "LD A, A", opcode: 0x7F, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ld_8bit(registers, Register::A, registers.a()) })) };

        instructions[0xC3] = Instruction{ name: "JP a16", opcode: 0xC3, length: 3, cycles: 2,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_bits: u8, upper_bits: u8| { Self::jmp(registers, lower_bits, upper_bits) })) };

        instructions[0x80] = Instruction{ name: "ADD A, B", opcode: 0x80, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.b()) })) };
        instructions[0x81] = Instruction{ name: "ADD A, C", opcode: 0x81, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.c()) })) };
        instructions[0x82] = Instruction{ name: "ADD A, D", opcode: 0x82, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.d()) })) };
        instructions[0x83] = Instruction{ name: "ADD A, E", opcode: 0x83, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(),  registers.e()) })) };
        instructions[0x84] = Instruction{ name: "ADD A, H", opcode: 0x84, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.h()) })) };
        instructions[0x85] = Instruction{ name: "ADD A, L", opcode: 0x85, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.l()) })) };
        instructions[0x86] = Instruction{ name: "ADD A, (HL)", opcode: 0x86, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::add(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x87] = Instruction{ name: "ADD A, A", opcode: 0x87, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.a()) })) };
        instructions[0x88] = Instruction{ name: "ADC A, B", opcode: 0x88, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.b()) })) };
        instructions[0x89] = Instruction{ name: "ADC A, C", opcode: 0x89, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.c()) })) };
        instructions[0x8A] = Instruction{ name: "ADC A, D", opcode: 0x8A, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.d()) })) };
        instructions[0x8B] = Instruction{ name: "ADC A, E", opcode: 0x8B, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(),  registers.e()) })) };
        instructions[0x8C] = Instruction{ name: "ADC A, H", opcode: 0x8C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.h()) })) };
        instructions[0x8D] = Instruction{ name: "ADC A, L", opcode: 0x8D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.l()) })) };
        instructions[0x8E] = Instruction{ name: "ADC A, (HL)", opcode: 0x8E, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::adc(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x8F] = Instruction{ name: "ADC A, A", opcode: 0x8F, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.a()) })) };

        instructions[0x90] = Instruction{ name: "SUB B", opcode: 0x90, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.b()) })) };
        instructions[0x91] = Instruction{ name: "SUB C", opcode: 0x91, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.c()) })) };
        instructions[0x92] = Instruction{ name: "SUB D", opcode: 0x92, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.d()) })) };
        instructions[0x93] = Instruction{ name: "SUB E", opcode: 0x93, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(),  registers.e()) })) };
        instructions[0x94] = Instruction{ name: "SUB H", opcode: 0x94, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.h()) })) };
        instructions[0x95] = Instruction{ name: "SUB L", opcode: 0x95, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.l()) })) };
        instructions[0x96] = Instruction{ name: "SUB (HL)", opcode: 0x96, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::sub(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x97] = Instruction{ name: "SUB A", opcode: 0x97, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.a()) })) };
        instructions[0x98] = Instruction{ name: "SBC B", opcode: 0x98, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.b()) })) };
        instructions[0x99] = Instruction{ name: "SBC C", opcode: 0x99, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.c()) })) };
        instructions[0x9A] = Instruction{ name: "SBC D", opcode: 0x9A, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.d()) })) };
        instructions[0x9B] = Instruction{ name: "SBC E", opcode: 0x9B, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(),  registers.e()) })) };
        instructions[0x9C] = Instruction{ name: "SBC H", opcode: 0x9C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.h()) })) };
        instructions[0x9D] = Instruction{ name: "SBC L", opcode: 0x9D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.l()) })) };
        instructions[0x9E] = Instruction{ name: "SBC (HL)", opcode: 0x9E, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::sbc(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x9F] = Instruction{ name: "SBC A", opcode: 0x9F, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.a()) })) };

        instructions[0xA0] = Instruction{ name: "AND B", opcode: 0xA0, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.b()) })) };
        instructions[0xA1] = Instruction{ name: "AND C", opcode: 0xA1, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.c()) })) };
        instructions[0xA2] = Instruction{ name: "AND D", opcode: 0xA2, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.d()) })) };
        instructions[0xA3] = Instruction{ name: "AND E", opcode: 0xA3, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(),  registers.e()) })) };
        instructions[0xA4] = Instruction{ name: "AND H", opcode: 0xA4, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.h()) })) };
        instructions[0xA5] = Instruction{ name: "AND L", opcode: 0xA5, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.l()) })) };
        instructions[0xA6] = Instruction{ name: "AND (HL)", opcode: 0xA6, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::and(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0xA7] = Instruction{ name: "AND A", opcode: 0xA7, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.a()) })) };
        instructions[0xA8] = Instruction{ name: "XOR B", opcode: 0xA8, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.b()) })) };
        instructions[0xA9] = Instruction{ name: "XOR C", opcode: 0xA9, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.c()) })) };
        instructions[0xAA] = Instruction{ name: "XOR D", opcode: 0xAA, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.d()) })) };
        instructions[0xAB] = Instruction{ name: "XOR E", opcode: 0xAB, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(),  registers.e()) })) };
        instructions[0xAC] = Instruction{ name: "XOR H", opcode: 0xAC, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.h()) })) };
        instructions[0xAD] = Instruction{ name: "XOR L", opcode: 0xAD, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.l()) })) };
        instructions[0xAE] = Instruction{ name: "XOR (HL)", opcode: 0xAE, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::xor(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0xAF] = Instruction{ name: "XOR A", opcode: 0xAF, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.a()) })) };

        instructions[0xB0] = Instruction{ name: "OR B", opcode: 0xB0, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.b()) })) };
        instructions[0xB1] = Instruction{ name: "OR C", opcode: 0xB1, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.c()) })) };
        instructions[0xB2] = Instruction{ name: "OR D", opcode: 0xB2, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.d()) })) };
        instructions[0xB3] = Instruction{ name: "OR E", opcode: 0xB3, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(),  registers.e()) })) };
        instructions[0xB4] = Instruction{ name: "OR H", opcode: 0xB4, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.h()) })) };
        instructions[0xB5] = Instruction{ name: "OR L", opcode: 0xB5, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.l()) })) };
        instructions[0xB6] = Instruction{ name: "OR (HL)", opcode: 0xB6, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::or(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0xB7] = Instruction{ name: "OR A", opcode: 0xB7, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.a()) })) };
        instructions[0xB8] = Instruction{ name: "CP B", opcode: 0xB8, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.b()) })) };
        instructions[0xB9] = Instruction{ name: "CP C", opcode: 0xB9, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.c()) })) };
        instructions[0xBA] = Instruction{ name: "CP D", opcode: 0xBA, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.d()) })) };
        instructions[0xBB] = Instruction{ name: "CP E", opcode: 0xBB, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(),  registers.e()) })) };
        instructions[0xBC] = Instruction{ name: "CP H", opcode: 0xBC, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.h()) })) };
        instructions[0xBD] = Instruction{ name: "CP L", opcode: 0xBD, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.l()) })) };
        instructions[0xBE] = Instruction{ name: "CP (HL)", opcode: 0xBE, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::cp(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0xBF] = Instruction{ name: "CP A", opcode: 0xBF, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.a()) })) };

        instructions[0xC1] = Instruction{ name: "POP BC", opcode: 0xC1, length: 1, cycles: 3,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::pop(mmu, registers, Register::BC) })) };
        instructions[0xC5] = Instruction{ name: "PUSH BC", opcode: 0xC5, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::push(mmu, registers, registers.bc()) })) };
        instructions[0xC6] = Instruction{ name: "ADD A, d8", opcode: 0xC6, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::add(registers, registers.a(), value) })) };
        instructions[0xCD] = Instruction{ name: "CALL a16", opcode: 0xCD, length: 3, cycles: 6,
            operation: Operation::Binary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, lower_byte: u8, higher_byte: u8| { Self::call(mmu, registers, lower_byte, higher_byte) })) };
        instructions[0xCE] = Instruction{ name: "ADC A, d8", opcode: 0xCE, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::adc(registers, registers.a(), value) })) };

        instructions[0xD1] = Instruction{ name: "POP DE", opcode: 0xD1, length: 1, cycles: 3,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::pop(mmu, registers, Register::DE) })) };
        instructions[0xD5] = Instruction{ name: "PUSH DE", opcode: 0xD5, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::push(mmu, registers, registers.de()) })) };
        instructions[0xD6] = Instruction{ name: "SUB A, d8", opcode: 0xD6, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::sub(registers, registers.a(), value) })) };
        instructions[0xDE] = Instruction{ name: "SBC A, d8", opcode: 0xDE, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::sbc(registers, registers.a(), value) })) };

        instructions[0xE1] = Instruction{ name: "POP HL", opcode: 0xE1, length: 1, cycles: 3,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::pop(mmu, registers, Register::HL) })) };
        instructions[0xE5] = Instruction{ name: "PUSH HL", opcode: 0xE5, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::push(mmu, registers, registers.hl()) })) };
        instructions[0xE6] = Instruction{ name: "AND A, d8", opcode: 0xE6, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::and(registers, registers.a(), value) })) };
        instructions[0xEE] = Instruction{ name: "XOR A, d8", opcode: 0xE, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::xor(registers, registers.a(), value) })) };

        instructions[0xF1] = Instruction{ name: "POP AF", opcode: 0xF1, length: 1, cycles: 3,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::pop(mmu, registers, Register::AF) })) };
        instructions[0xF3] = Instruction{ name: "DI", opcode: 0xF3, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, _| { mmu.set_ime(0) })) };
        instructions[0xF5] = Instruction{ name: "PUSH AF", opcode: 0xF5, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::push(mmu, registers, registers.af()) })) };
        instructions[0xF6] = Instruction{ name: "OR A, d8", opcode: 0xF6, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::or(registers, registers.a(), value) })) };
        instructions[0xFE] = Instruction{ name: "CP d8", opcode: 0xF8, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::cp(registers, registers.a(), value) })) };

        InstructionSet { instructions, mmu }
    }

    pub fn fetch_instruction(&self, opcode: u8) -> Instruction {
        self.instructions[opcode as usize].clone()
    }

    // Increment the contents of a register by 1
    // Flags: Z 0 8-bit -
    fn inc_8bit(registers: &mut Registers, register: Register) {
        let original_value = registers.get_8bit_register(register.clone());
        let new_value = original_value + 1;
        registers.set_8bit_register(register, new_value);

        if new_value == 0 {
            registers.set_zero_flag(true);
        }

        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(carry_check_add_8bit(original_value, 1));
    }

    // Decrement the contents of a register by 1
    // Flags: Z 1 8-bit -
    fn dec_8bit(registers: &mut Registers, register: Register) {
        let original_value = registers.get_8bit_register(register.clone());
        let new_value = original_value - 1;
        registers.set_8bit_register(register, new_value);

        registers.set_zero_flag(new_value == 0);
        registers.set_subtraction_flag(true);
        registers.set_half_carry_flag(carry_check_sub_8bit(original_value, 1));
    }

    // Increment the contents of a register pair by 1
    // Flags: - - - -
    fn inc_16bit(registers: &mut Registers, register: Register) {
        let original_value = registers.get_16bit_register(register.clone());
        let new_value = original_value + 1;
        registers.set_16bit_register(register, new_value);
    }

    // Decrements the contents of a register pair by 1
    // Flags: - - - -
    fn dec_16bit(registers: &mut Registers, register: Register) {
        let original_value = registers.get_16bit_register(register.clone());
        let new_value = original_value - 1;
        registers.set_16bit_register(register, new_value);
    }

    // Loads the value of address in the program counter
    // Flags: - - - -
    fn jmp(registers: &mut Registers, lower_order_byte: u8, higher_order_byte: u8) {
        let address = concatenate_bytes(lower_order_byte, higher_order_byte);
        registers.set_pc(address);
    }

    // Add two values and store the results in register A
    // Flags: Z 0 8-bit 8-bit
    fn add(registers: &mut Registers, left_operator: u8, right_operator: u8) {
        let sum = left_operator + right_operator;

        registers.set_8bit_register(Register::A, sum);

        registers.set_zero_flag(sum == 0);
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(carry_check_add_8bit(left_operator, right_operator));
        registers.set_carry_flag((left_operator as u16 + right_operator as u16) > 0xFF);
    }

    // Add two values along with the carry flag and store the content in register A
    // Flags: Z 0 8-bit 8-bit
    fn adc(registers: &mut Registers, left_operator: u8, right_operator: u8) {
        let sum = left_operator + right_operator + registers.carry_flag() as u8;

        registers.set_8bit_register(Register::A, sum);

        registers.set_zero_flag(sum == 0);
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(carry_check_add_8bit(left_operator, right_operator));
        registers.set_carry_flag((left_operator as u16 + right_operator as u16) > 0xFF);
    }

    // Subtract two values and store the result in register A
    // Flags: Z 1 8-bit 8-bit
    fn sub(registers: &mut Registers, left_operator: u8, right_operator: u8) {
        let difference = left_operator - right_operator;

        registers.set_8bit_register(Register::A, difference);

        registers.set_zero_flag(difference == 0);
        registers.set_subtraction_flag(true);
        registers.set_half_carry_flag(carry_check_sub_8bit(left_operator, right_operator));
        registers.set_carry_flag((left_operator as i16 - right_operator as i16) < 0);
    }

    // Subtract two values and the carry flag and store the result in register A
    // Flags: Z 1 8-bit 8-bit
    fn sbc(registers: &mut Registers, left_operator: u8, right_operator: u8) {
        let difference = left_operator - right_operator - registers.carry_flag() as u8;

        registers.set_8bit_register(Register::A, difference);

        registers.set_zero_flag(difference == 0);
        registers.set_subtraction_flag(true);
        registers.set_half_carry_flag(carry_check_sub_8bit(left_operator, right_operator));
        registers.set_carry_flag((left_operator as i16 - right_operator as i16) < 0);
    }

    // Take the logical AND for each bit of the operands and store the result in register A
    // Flags: Z 0 1 0
    fn and(registers: &mut Registers, left_operator: u8, right_operator: u8) {
        let result = left_operator & right_operator;
        registers.set_8bit_register(Register::A, result);

        registers.set_zero_flag(result == 0);
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(true);
        registers.set_carry_flag(false);
    }

    // Take the logical OR for each bit of the operands and store the result in register A
    // Flags: Z 0 0 0
    fn or(registers: &mut Registers, left_operator: u8, right_operator: u8) {
        let result = left_operator | right_operator;
        registers.set_8bit_register(Register::A, result);

        registers.set_zero_flag(result == 0);
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(false);
        registers.set_carry_flag(false);
    }

    // Take the logical XOR for each bit of the operands and store the result in register A
    // Flags: Z 0 0 0
    fn xor(registers: &mut Registers, left_operator: u8, right_operator: u8) {
        let result = left_operator ^ right_operator;
        registers.set_8bit_register(Register::A, result);

        registers.set_zero_flag(result == 0);
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(false);
        registers.set_carry_flag(false);
    }

    // Compare the contents of register C and the contents of register A by calculating A - C, and set the Z flag if they are equal.
    // Flags: Z 1 8-bit 8-bit
    fn cp(registers: &mut Registers, left_operator: u8, right_operator: u8) {
        let difference = left_operator - right_operator;

        registers.set_zero_flag(difference == 0);
        registers.set_subtraction_flag(true);
        registers.set_half_carry_flag(carry_check_sub_8bit(left_operator, right_operator));
        registers.set_carry_flag((left_operator as i16 - right_operator as i16) < 0);
    }

    // Load the value into the given register
    // Flags:: - - - -
    fn ld_8bit(registers: &mut Registers, register: Register, value: u8) {
        registers.set_8bit_register(register, value);
    }

    fn ld_8bit_mem(mmu: &mut Mmu, address: u16, value: u8) {
        mmu.write_byte(address as usize, value).unwrap()
    }

    // Load the 2 bytes of immediate data in the given register
    // Flags: - - - -
    fn ld_16bit(registers: &mut Registers, register: Register, lower_byte: u8, higher_byte: u8) {
        registers.set_16bit_register(register, concatenate_bytes(lower_byte, higher_byte));
    }

    // Unconditional function call to the absolute address specified by the 16-bit operand nn
    // Flags: - - - -
    fn call(mmu: &mut Mmu, registers: &mut Registers, lower_byte: u8, higher_byte: u8) {
        let jump_address = concatenate_bytes(lower_byte, higher_byte);
        let (return_lower, return_higher) = split_2bytes(registers.pc());

        registers.decrement_sp();
        mmu.write_byte(registers.sp() as usize, return_lower).unwrap();
        registers.decrement_sp();
        mmu.write_byte(registers.sp() as usize, return_higher).unwrap();

        registers.set_pc(jump_address)
    }

    // Jump n steps from the current pc
    // Flags: - - - -
    fn jr(registers: &mut Registers, steps: u8) {
        let current_pc = registers.pc() as i16;
        let steps = (steps as i8) as i16; // Steps is signed... source of a nasty bug

        registers.set_pc((current_pc + steps) as u16);
    }

    // Push a value on the stack
    // Flags: - - - -
    fn push(mmu: &mut Mmu, registers: &mut Registers, value: u16) {
        let (value_lower, value_higher) = split_2bytes(value);

        registers.decrement_sp();
        mmu.write_byte(registers.sp() as usize, value_lower).unwrap();
        registers.decrement_sp();
        mmu.write_byte(registers.sp() as usize, value_higher).unwrap();
    }

    // Pop a value from the stack and store it in the given register
    // Flags: - - - -
    fn pop(mmu: &mut Mmu, registers: &mut Registers, register: Register) {
        let value_higher = mmu.read_byte(registers.sp() as usize).unwrap();
        registers.increment_sp();
        let value_lower = mmu.read_byte(registers.sp() as usize).unwrap();
        registers.increment_sp();

        registers.set_16bit_register(register, concatenate_bytes(value_lower, value_higher))
    }
}