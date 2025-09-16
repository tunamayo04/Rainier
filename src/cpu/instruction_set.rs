use std::cell::{RefCell};
use std::{fmt, mem};
use std::mem::MaybeUninit;
use std::rc::Rc;
use crate::bit_utils::{carry_check_add_16bit, carry_check_add_8bit, carry_check_sub_8bit, concatenate_bytes, split_2bytes};
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
    pub name: String,
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
    instructions_8bit: [Instruction; 256],
    instructions_16bit: [Instruction; 256],

    mmu: Rc<RefCell<Mmu>>,
}

impl InstructionSet {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> InstructionSet {
        let mut instructions_8bit: [MaybeUninit<Instruction>; 256] = unsafe {MaybeUninit::uninit().assume_init() };
        let mut instructions_16bit: [MaybeUninit<Instruction>; 256] = unsafe {MaybeUninit::uninit().assume_init() };

        for i in 0..256 {
            instructions_8bit[i] = MaybeUninit::new(Instruction::default());
            instructions_16bit[i] = MaybeUninit::new(Instruction::default());
        }

        // region 8-bit opcodes
        let mut instructions_8bit: [Instruction; 256] = unsafe { mem::transmute(instructions_8bit) };

        instructions_8bit[0x00] = Instruction{ name: String::from("NOP"), opcode: 0x00, length: 1, cycles: 1, operation: Operation::None, };
        instructions_8bit[0x01] = Instruction{ name: String::from("LD BC, d16"), opcode: 0x01, length: 3, cycles: 3,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_byte: u8, higher_byte: u8| { Self::ld_16bit(registers, Register::BC, lower_byte, higher_byte) })) };
        instructions_8bit[0x02] = Instruction{ name: String::from("LD (BC), A"), opcode: 0x02, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit_mem(mmu, registers.bc(), registers.a()) })) } ;
        instructions_8bit[0x03] = Instruction{ name: String::from("INC BC"), opcode: 0x03, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::BC ) })) };
        instructions_8bit[0x04] = Instruction{ name: String::from("INC B"), opcode: 0x04, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::B) })) };
        instructions_8bit[0x05] = Instruction{ name: String::from("DEC B"), opcode: 0x05, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::B) })) };
        instructions_8bit[0x06] = Instruction{ name: String::from("LD B, d8"), opcode: 0x06, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::B, value) })) } ;
        instructions_8bit[0x08] = Instruction{ name: String::from("LD (a16), SP"), opcode: 0x08, length: 3, cycles: 5,
            operation: Operation::Binary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, lower_byte: u8, upper_byte: u8| {
                let (lower_sp, higher_sp) = split_2bytes(registers.sp());
                let address = concatenate_bytes(lower_byte, upper_byte);
                Self::ld_8bit_mem(mmu, address, lower_sp);
                Self::ld_8bit_mem(mmu, address + 1, higher_sp);
            })) } ;
        instructions_8bit[0x09] = Instruction{ name: String::from("ADD HL, BC"), opcode: 0x09, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add_16bit(registers, Register::HL, registers.bc()) })) } ;
        instructions_8bit[0x0A] = Instruction{ name: String::from("LD A, (BC)"), opcode: 0x0A, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit(registers, Register::A, mmu.read_byte(registers.bc() as usize).unwrap()) })) };
        instructions_8bit[0x0B] = Instruction{ name: String::from("DEC BC"), opcode: 0x0B, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::dec_16bit(registers, Register::BC) })) };
        instructions_8bit[0x0C] = Instruction{ name: String::from("INC C"), opcode: 0x0C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::C) })) };
        instructions_8bit[0x0D] = Instruction{ name: String::from("DEC C"), opcode: 0x0D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::C) })) };
        instructions_8bit[0x0E] = Instruction{ name: String::from("LD C, d8"), opcode: 0x0E, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::C, value) })) } ;

        instructions_8bit[0x11] = Instruction{ name: String::from("LD DE, d16"), opcode: 0x11, length: 3, cycles: 3,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_byte: u8, higher_byte: u8| { Self::ld_16bit(registers, Register::DE, lower_byte, higher_byte) })) };
        instructions_8bit[0x12] = Instruction{ name: String::from("LD (DE), A"), opcode: 0x12, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit_mem(mmu, registers.de(), registers.a()) })) } ;
        instructions_8bit[0x13] = Instruction{ name: String::from("INC DE"), opcode: 0x13, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::DE ) })) };
        instructions_8bit[0x14] = Instruction{ name: String::from("INC D"), opcode: 0x14, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::D) })) };
        instructions_8bit[0x15] = Instruction{ name: String::from("DEC D"), opcode: 0x15, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::D) })) };
        instructions_8bit[0x16] = Instruction{ name: String::from("LD D, d8"), opcode: 0x16, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::D, value) })) } ;
        instructions_8bit[0x18] = Instruction{ name: String::from("JR s8"), opcode: 0x18, length: 2, cycles: 3,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::jr(registers, value) })) } ;
        instructions_8bit[0x19] = Instruction{ name: String::from("ADD HL, DE"), opcode: 0x19, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add_16bit(registers, Register::HL, registers.de()) })) } ;
        instructions_8bit[0x1A] = Instruction{ name: String::from("LD A, (DE)"), opcode: 0x1A, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit(registers, Register::A, mmu.read_byte(registers.de() as usize).unwrap()) })) };
        instructions_8bit[0x1B] = Instruction{ name: String::from("DEC DE"), opcode: 0x1B, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::dec_16bit(registers, Register::DE) })) };
        instructions_8bit[0x1C] = Instruction{ name: String::from("INC E"), opcode: 0x1C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::E) })) };
        instructions_8bit[0x1D] = Instruction{ name: String::from("DEC E"), opcode: 0x1D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::E) })) };
        instructions_8bit[0x1E] = Instruction{ name: String::from("LD E, d8"), opcode: 0x1E, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::E, value) })) } ;

        instructions_8bit[0x20] = Instruction{ name: String::from("JR NZ, s8"), opcode: 0x20, length: 2, cycles: 3, // TODO: Check the variable cycles implementation
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, steps: u8| { if registers.zero_flag() == false { Self::jr(registers, steps) } })) };
        instructions_8bit[0x21] = Instruction{ name: String::from("LD HL, d16"), opcode: 0x21, length: 3, cycles: 3,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_byte: u8, higher_byte: u8| { Self::ld_16bit(registers, Register::HL, lower_byte, higher_byte) })) };
        instructions_8bit[0x22] = Instruction{ name: String::from("LD (HL+), A"), opcode: 0x22, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| {
                Self::ld_8bit_mem(mmu, registers.hl(), registers.a());
                registers.set_hl(registers.hl() + 1) })) } ;
        instructions_8bit[0x23] = Instruction{ name: String::from("INC HL"), opcode: 0x23, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::HL ) })) };
        instructions_8bit[0x24] = Instruction{ name: String::from("INC H"), opcode: 0x24, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::H) })) };
        instructions_8bit[0x25] = Instruction{ name: String::from("DEC H"), opcode: 0x25, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::H) })) };
        instructions_8bit[0x26] = Instruction{ name: String::from("LD H, d8"), opcode: 0x26, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::H, value) })) } ;
        instructions_8bit[0x28] = Instruction{ name: String::from("JR Z, s8"), opcode: 0x28, length: 2, cycles: 3,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { if registers.zero_flag() { Self::jr(registers, value) } })) } ;
        instructions_8bit[0x29] = Instruction{ name: String::from("ADD HL, HL"), opcode: 0x29, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add_16bit(registers, Register::HL, registers.hl()) })) } ;
        instructions_8bit[0x2A] = Instruction{ name: String::from("LD A, (HL+)"), opcode: 0x2A, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| {
                Self::ld_8bit(registers, Register::A, mmu.read_byte(registers.hl() as usize).unwrap());
                registers.set_hl(registers.hl() + 1)})) };
        instructions_8bit[0x3B] = Instruction{ name: String::from("DEC HL"), opcode: 0x3B, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::dec_16bit(registers, Register::HL) })) };
        instructions_8bit[0x2C] = Instruction{ name: String::from("INC L"), opcode: 0x2C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::L) })) };
        instructions_8bit[0x2D] = Instruction{ name: String::from("DEC L"), opcode: 0x2D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::L) })) };
        instructions_8bit[0x2E] = Instruction{ name: String::from("LD L, d8"), opcode: 0x2E, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::L, value) })) } ;
        instructions_8bit[0x2F] = Instruction{ name: String::from("CPL"), opcode: 0x2F, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cpl(registers) })) } ;

        instructions_8bit[0x30] = Instruction{ name: String::from("JR NC, s8"), opcode: 0x30, length: 2, cycles: 3, // TODO: Check the variable cycles implementation
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, steps: u8| { if registers.carry_flag() == false { Self::jr(registers, steps) } })) };
        instructions_8bit[0x31] = Instruction{ name: String::from("LD SP, d16"), opcode: 0x31, length: 3, cycles: 3,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_byte: u8, higher_byte: u8| { Self::ld_16bit(registers, Register::SP, lower_byte, higher_byte) })) };
        instructions_8bit[0x32] = Instruction{ name: String::from("LD (HL-), A"), opcode: 0x32, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| {
                Self::ld_8bit_mem(mmu, registers.hl(), registers.a());
                registers.set_hl(registers.hl() - 1) })) } ;
        instructions_8bit[0x33] = Instruction{ name: String::from("INC SP"), opcode: 0x33, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::SP ) })) };
        instructions_8bit[0x34] = Instruction{ name: String::from("INC (HL)"), opcode: 0x34, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::inc_mem(registers, mmu, registers.hl() as usize) })) };
        instructions_8bit[0x35] = Instruction{ name: String::from("DEC (HL)"), opcode: 0x35, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::dec_mem(registers, mmu, registers.hl() as usize) })) };
        instructions_8bit[0x36] = Instruction{ name: String::from("LD (HL), d8"), opcode: 0x36, length: 2, cycles: 3,
            operation: Operation::Unary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, value: u8| { Self::ld_8bit_mem(mmu, registers.hl(), value) })) } ;
        instructions_8bit[0x38] = Instruction{ name: String::from("JRCZ, s8"), opcode: 0x38, length: 2, cycles: 3,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { if registers.carry_flag() { Self::jr(registers, value) } })) } ;
        instructions_8bit[0x39] = Instruction{ name: String::from("ADD HL, SP"), opcode: 0x39, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add_16bit(registers, Register::HL, registers.sp()) })) } ;
        instructions_8bit[0x3A] = Instruction{ name: String::from("LD A, (HL-)"), opcode: 0x3A, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| {
                Self::ld_8bit(registers, Register::A, mmu.read_byte(registers.hl() as usize).unwrap());
                registers.set_hl(registers.hl() - 1)})) };
        instructions_8bit[0x3B] = Instruction{ name: String::from("DEC SP"), opcode: 0x3B, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::dec_16bit(registers, Register::SP) })) };
        instructions_8bit[0x3C] = Instruction{ name: String::from("INC A"), opcode: 0x3C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::A) })) };
        instructions_8bit[0x3D] = Instruction{ name: String::from("DEC A"), opcode: 0x3D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::A) })) };
        instructions_8bit[0x3E] = Instruction{ name: String::from("LD A, d8"), opcode: 0x3E, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::ld_8bit(registers, Register::A, value) })) } ;
        instructions_8bit[0x3F] = Instruction{ name: String::from("CCF"), opcode: 0x3F, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::ccf(registers) })) } ;


        let source_regs: [(&str, fn(&Registers) -> u8); 8] = [
            ("B", Registers::b),
            ("C", Registers::c),
            ("D", Registers::d),
            ("E", Registers::e),
            ("H", Registers::h),
            ("L", Registers::l),
            ("(HL)", |_| panic!("special case")), // handled separately
            ("A", Registers::a),
        ];

        let destination_regs: [(&str, Register); 8] = [
            ("B", Register::B),
            ("C", Register::C),
            ("D", Register::D),
            ("E", Register::E),
            ("H", Register::H),
            ("L", Register::L),
            ("(HL)", Register::HL), // handled separately
            ("A", Register::A),
        ];

        // LD
        for (i, (destination_name, destination_accessor)) in destination_regs.iter().enumerate() {
            for (j, (source_name, source_accessor)) in source_regs.iter().enumerate() {
                let name = format!("LD {}, {}", destination_name, source_name);
                let opcode = 0x40 + i as u8 * 8 + j as u8;
                let cycles = if name == "(HL)" { 2 } else { 1 };
                let source_accessor = *source_accessor;
                let destination_accessor = *destination_accessor;

                let operation = if *destination_name == "(HL)" {
                    if *source_name == "(HL)" {
                        continue;
                    }
                    else {
                        Operation::Nullary(Rc::new(move |mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit_mem(mmu, registers.hl(), source_accessor(registers)) }))
                    }
                } else {
                    if *source_name == "(HL)" {
                        Operation::Nullary(Rc::new(move |mmu: &mut Mmu, registers: &mut Registers| { Self::ld_8bit(registers, destination_accessor, mmu.read_byte(source_accessor(registers) as usize).unwrap()) }))
                    } else {
                        Operation::Nullary(Rc::new(move |_, registers: &mut Registers| { Self::ld_8bit(registers, destination_accessor, source_accessor(registers)) }))
                    }
                };

                instructions_8bit[opcode as usize] = Instruction { name, opcode, length: 1, cycles, operation };
            }
        }

        instructions_8bit[0x80] = Instruction{ name: String::from("ADD A, B"), opcode: 0x80, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.b()) })) };
        instructions_8bit[0x81] = Instruction{ name: String::from("ADD A, C"), opcode: 0x81, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.c()) })) };
        instructions_8bit[0x82] = Instruction{ name: String::from("ADD A, D"), opcode: 0x82, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.d()) })) };
        instructions_8bit[0x83] = Instruction{ name: String::from("ADD A, E"), opcode: 0x83, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(),  registers.e()) })) };
        instructions_8bit[0x84] = Instruction{ name: String::from("ADD A, H"), opcode: 0x84, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.h()) })) };
        instructions_8bit[0x85] = Instruction{ name: String::from("ADD A, L"), opcode: 0x85, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.l()) })) };
        instructions_8bit[0x86] = Instruction{ name: String::from("ADD A, (HL)"), opcode: 0x86, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::add(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions_8bit[0x87] = Instruction{ name: String::from("ADD A, A"), opcode: 0x87, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.a()) })) };
        instructions_8bit[0x88] = Instruction{ name: String::from("ADC A, B"), opcode: 0x88, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.b()) })) };
        instructions_8bit[0x89] = Instruction{ name: String::from("ADC A, C"), opcode: 0x89, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.c()) })) };
        instructions_8bit[0x8A] = Instruction{ name: String::from("ADC A, D"), opcode: 0x8A, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.d()) })) };
        instructions_8bit[0x8B] = Instruction{ name: String::from("ADC A, E"), opcode: 0x8B, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(),  registers.e()) })) };
        instructions_8bit[0x8C] = Instruction{ name: String::from("ADC A, H"), opcode: 0x8C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.h()) })) };
        instructions_8bit[0x8D] = Instruction{ name: String::from("ADC A, L"), opcode: 0x8D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.l()) })) };
        instructions_8bit[0x8E] = Instruction{ name: String::from("ADC A, (HL)"), opcode: 0x8E, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::adc(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions_8bit[0x8F] = Instruction{ name: String::from("ADC A, A"), opcode: 0x8F, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.a()) })) };

        instructions_8bit[0x90] = Instruction{ name: String::from("SUB B"), opcode: 0x90, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.b()) })) };
        instructions_8bit[0x91] = Instruction{ name: String::from("SUB C"), opcode: 0x91, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.c()) })) };
        instructions_8bit[0x92] = Instruction{ name: String::from("SUB D"), opcode: 0x92, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.d()) })) };
        instructions_8bit[0x93] = Instruction{ name: String::from("SUB E"), opcode: 0x93, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(),  registers.e()) })) };
        instructions_8bit[0x94] = Instruction{ name: String::from("SUB H"), opcode: 0x94, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.h()) })) };
        instructions_8bit[0x95] = Instruction{ name: String::from("SUB L"), opcode: 0x95, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.l()) })) };
        instructions_8bit[0x96] = Instruction{ name: String::from("SUB (HL)"), opcode: 0x96, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::sub(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions_8bit[0x97] = Instruction{ name: String::from("SUB A"), opcode: 0x97, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.a()) })) };
        instructions_8bit[0x98] = Instruction{ name: String::from("SBC B"), opcode: 0x98, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.b()) })) };
        instructions_8bit[0x99] = Instruction{ name: String::from("SBC C"), opcode: 0x99, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.c()) })) };
        instructions_8bit[0x9A] = Instruction{ name: String::from("SBC D"), opcode: 0x9A, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.d()) })) };
        instructions_8bit[0x9B] = Instruction{ name: String::from("SBC E"), opcode: 0x9B, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(),  registers.e()) })) };
        instructions_8bit[0x9C] = Instruction{ name: String::from("SBC H"), opcode: 0x9C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.h()) })) };
        instructions_8bit[0x9D] = Instruction{ name: String::from("SBC L"), opcode: 0x9D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.l()) })) };
        instructions_8bit[0x9E] = Instruction{ name: String::from("SBC (HL)"), opcode: 0x9E, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::sbc(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions_8bit[0x9F] = Instruction{ name: String::from("SBC A"), opcode: 0x9F, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.a()) })) };

        instructions_8bit[0xA0] = Instruction{ name: String::from("AND B"), opcode: 0xA0, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.b()) })) };
        instructions_8bit[0xA1] = Instruction{ name: String::from("AND C"), opcode: 0xA1, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.c()) })) };
        instructions_8bit[0xA2] = Instruction{ name: String::from("AND D"), opcode: 0xA2, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.d()) })) };
        instructions_8bit[0xA3] = Instruction{ name: String::from("AND E"), opcode: 0xA3, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(),  registers.e()) })) };
        instructions_8bit[0xA4] = Instruction{ name: String::from("AND H"), opcode: 0xA4, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.h()) })) };
        instructions_8bit[0xA5] = Instruction{ name: String::from("AND L"), opcode: 0xA5, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.l()) })) };
        instructions_8bit[0xA6] = Instruction{ name: String::from("AND (HL)"), opcode: 0xA6, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::and(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions_8bit[0xA7] = Instruction{ name: String::from("AND A"), opcode: 0xA7, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.a()) })) };
        instructions_8bit[0xA8] = Instruction{ name: String::from("XOR B"), opcode: 0xA8, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.b()) })) };
        instructions_8bit[0xA9] = Instruction{ name: String::from("XOR C"), opcode: 0xA9, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.c()) })) };
        instructions_8bit[0xAA] = Instruction{ name: String::from("XOR D"), opcode: 0xAA, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.d()) })) };
        instructions_8bit[0xAB] = Instruction{ name: String::from("XOR E"), opcode: 0xAB, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(),  registers.e()) })) };
        instructions_8bit[0xAC] = Instruction{ name: String::from("XOR H"), opcode: 0xAC, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.h()) })) };
        instructions_8bit[0xAD] = Instruction{ name: String::from("XOR L"), opcode: 0xAD, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.l()) })) };
        instructions_8bit[0xAE] = Instruction{ name: String::from("XOR (HL)"), opcode: 0xAE, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::xor(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions_8bit[0xAF] = Instruction{ name: String::from("XOR A"), opcode: 0xAF, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.a()) })) };

        instructions_8bit[0xB0] = Instruction{ name: String::from("OR B"), opcode: 0xB0, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.b()) })) };
        instructions_8bit[0xB1] = Instruction{ name: String::from("OR C"), opcode: 0xB1, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.c()) })) };
        instructions_8bit[0xB2] = Instruction{ name: String::from("OR D"), opcode: 0xB2, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.d()) })) };
        instructions_8bit[0xB3] = Instruction{ name: String::from("OR E"), opcode: 0xB3, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(),  registers.e()) })) };
        instructions_8bit[0xB4] = Instruction{ name: String::from("OR H"), opcode: 0xB4, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.h()) })) };
        instructions_8bit[0xB5] = Instruction{ name: String::from("OR L"), opcode: 0xB5, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.l()) })) };
        instructions_8bit[0xB6] = Instruction{ name: String::from("OR (HL)"), opcode: 0xB6, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::or(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions_8bit[0xB7] = Instruction{ name: String::from("OR A"), opcode: 0xB7, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.a()) })) };
        instructions_8bit[0xB8] = Instruction{ name: String::from("CP B"), opcode: 0xB8, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.b()) })) };
        instructions_8bit[0xB9] = Instruction{ name: String::from("CP C"), opcode: 0xB9, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.c()) })) };
        instructions_8bit[0xBA] = Instruction{ name: String::from("CP D"), opcode: 0xBA, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.d()) })) };
        instructions_8bit[0xBB] = Instruction{ name: String::from("CP E"), opcode: 0xBB, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(),  registers.e()) })) };
        instructions_8bit[0xBC] = Instruction{ name: String::from("CP H"), opcode: 0xBC, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.h()) })) };
        instructions_8bit[0xBD] = Instruction{ name: String::from("CP L"), opcode: 0xBD, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.l()) })) };
        instructions_8bit[0xBE] = Instruction{ name: String::from("CP (HL)"), opcode: 0xBE, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::cp(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions_8bit[0xBF] = Instruction{ name: String::from("CP A"), opcode: 0xBF, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.a()) })) };

        instructions_8bit[0xC0] = Instruction{ name: String::from("RET NZ"), opcode: 0xC0, length: 1, cycles: 5,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { if !registers.zero_flag() { Self::ret(mmu, registers) } })) };
        instructions_8bit[0xC1] = Instruction{ name: String::from("POP BC"), opcode: 0xC1, length: 1, cycles: 3,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::pop(mmu, registers, Register::BC) })) };
        instructions_8bit[0xC2] = Instruction{ name: String::from("JP NZ, a16"), opcode: 0xC2, length: 3, cycles: 4,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_bits: u8, upper_bits: u8| { if !registers.zero_flag() { Self::jmp(registers, lower_bits, upper_bits) } })) };
        instructions_8bit[0xC3] = Instruction{ name: String::from("JP a16"), opcode: 0xC3, length: 3, cycles: 2,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_bits: u8, upper_bits: u8| { Self::jmp(registers, lower_bits, upper_bits) })) };
        instructions_8bit[0xC4] = Instruction{ name: String::from("CALL NZ, a16"), opcode: 0xC4, length: 3, cycles: 6,
            operation: Operation::Binary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, lower_bits: u8, upper_bits: u8| { if !registers.zero_flag() { Self::call(mmu, registers, lower_bits, upper_bits) } })) };
        instructions_8bit[0xC5] = Instruction{ name: String::from("PUSH BC"), opcode: 0xC5, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::push(mmu, registers, registers.bc()) })) };
        instructions_8bit[0xC6] = Instruction{ name: String::from("ADD A, d8"), opcode: 0xC6, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::add(registers, registers.a(), value) })) };
        instructions_8bit[0xC7] = Instruction{ name: String::from("RST 0"), opcode: 0xC7, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::call(mmu, registers, 0, 0) })) };
        instructions_8bit[0xC8] = Instruction{ name: String::from("RET Z"), opcode: 0xC8, length: 1, cycles: 5, // TODO: Handle variable cycles
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { if registers.zero_flag() { Self::ret(mmu, registers) } })) };
        instructions_8bit[0xC9] = Instruction{ name: String::from("RET"), opcode: 0xC9, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::ret(mmu, registers) })) };
        instructions_8bit[0xCA] = Instruction{ name: String::from("JP Z, a16"), opcode: 0xCA, length: 3, cycles: 4,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_bits: u8, upper_bits: u8| { if registers.zero_flag() { Self::jmp(registers, lower_bits, upper_bits) } })) };
        instructions_8bit[0xCC] = Instruction{ name: String::from("CALL Z, a16"), opcode: 0xCC, length: 3, cycles: 6,
            operation: Operation::Binary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, lower_byte: u8, higher_byte: u8| { if registers.zero_flag() { Self::call(mmu, registers, lower_byte, higher_byte) } })) };
        instructions_8bit[0xCD] = Instruction{ name: String::from("CALL a16"), opcode: 0xCD, length: 3, cycles: 6,
            operation: Operation::Binary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, lower_byte: u8, higher_byte: u8| { Self::call(mmu, registers, lower_byte, higher_byte) })) };
        instructions_8bit[0xCE] = Instruction{ name: String::from("ADC A, d8"), opcode: 0xCE, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::adc(registers, registers.a(), value) })) };
        instructions_8bit[0xCF] = Instruction{ name: String::from("RST 1"), opcode: 0xC7, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::call(mmu, registers, 0, 0x08) })) };

        instructions_8bit[0xD0] = Instruction{ name: String::from("RET NC"), opcode: 0xD0, length: 1, cycles: 5,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { if !registers.carry_flag() { Self::ret(mmu, registers) } })) };
        instructions_8bit[0xD1] = Instruction{ name: String::from("POP DE"), opcode: 0xD1, length: 1, cycles: 3,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::pop(mmu, registers, Register::DE) })) };
        instructions_8bit[0xD2] = Instruction{ name: String::from("JP NC, a16"), opcode: 0xD2, length: 3, cycles: 4,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_bits: u8, upper_bits: u8| { if !registers.carry_flag() { Self::jmp(registers, lower_bits, upper_bits) } })) };
        instructions_8bit[0xC4] = Instruction{ name: String::from("CALL NC, a16"), opcode: 0xD4, length: 3, cycles: 6,
            operation: Operation::Binary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, lower_bits: u8, upper_bits: u8| { if !registers.carry_flag() { Self::call(mmu, registers, lower_bits, upper_bits) } })) };
        instructions_8bit[0xD5] = Instruction{ name: String::from("PUSH DE"), opcode: 0xD5, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::push(mmu, registers, registers.de()) })) };
        instructions_8bit[0xD6] = Instruction{ name: String::from("SUB A, d8"), opcode: 0xD6, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::sub(registers, registers.a(), value) })) };
        instructions_8bit[0xD7] = Instruction{ name: String::from("RST 2"), opcode: 0xD7, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::call(mmu, registers, 0x10, 0) })) };
        instructions_8bit[0xD8] = Instruction{ name: String::from("RET C"), opcode: 0xD8, length: 1, cycles: 5, // TODO: Handle variable cycles
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { if registers.carry_flag() { Self::ret(mmu, registers) } })) };
        instructions_8bit[0xDA] = Instruction{ name: String::from("JP C, a16"), opcode: 0xDA, length: 3, cycles: 4,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_bits: u8, upper_bits: u8| { if registers.carry_flag() { Self::jmp(registers, lower_bits, upper_bits) } })) };
        instructions_8bit[0xDC] = Instruction{ name: String::from("CALL C, a16"), opcode: 0xDC, length: 3, cycles: 6,
            operation: Operation::Binary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, lower_byte: u8, higher_byte: u8| { if registers.carry_flag() { Self::call(mmu, registers, lower_byte, higher_byte) } })) };
        instructions_8bit[0xDE] = Instruction{ name: String::from("SBC A, d8"), opcode: 0xDE, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::sbc(registers, registers.a(), value) })) };
        instructions_8bit[0xDF] = Instruction{ name: String::from("RST 3"), opcode: 0xDF, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::call(mmu, registers, 0, 0x18) })) };

        instructions_8bit[0xE0] = Instruction{ name: String::from("LD (a8), A"), opcode: 0xE0, length: 2, cycles: 4,
            operation: Operation::Unary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, lower_byte: u8| {
                let address = concatenate_bytes(lower_byte, 0xFF);
                Self::ld_8bit_mem(mmu, address, registers.a()) }))
        };
        instructions_8bit[0xE1] = Instruction{ name: String::from("POP HL"), opcode: 0xE1, length: 1, cycles: 3,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::pop(mmu, registers, Register::HL) })) };
        instructions_8bit[0xE2] = Instruction{ name: String::from("LD (C), A"), opcode: 0xE2, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| {
                let address = concatenate_bytes(registers.c(), 0xFF);
                Self::ld_8bit_mem(mmu, address, registers.a()) }))
        };
        instructions_8bit[0xE5] = Instruction{ name: String::from("PUSH HL"), opcode: 0xE5, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::push(mmu, registers, registers.hl()) })) };
        instructions_8bit[0xE6] = Instruction{ name: String::from("AND A, d8"), opcode: 0xE6, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::and(registers, registers.a(), value) })) };
        instructions_8bit[0xE7] = Instruction{ name: String::from("RST 4"), opcode: 0xE7, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::call(mmu, registers, 0, 0x20) })) };
        instructions_8bit[0xE8] = Instruction{ name: String::from("ADD SP, s8"), opcode: 0xE8, length: 1, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::add_8bit_signed(registers, Register::HL, value as i8) })) } ;
        instructions_8bit[0xEA] = Instruction{ name: String::from("LD (a16), A"), opcode: 0xEA, length: 3, cycles: 4,
            operation: Operation::Binary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, lower_byte: u8, higher_byte: u8| {
                let address = concatenate_bytes(lower_byte, higher_byte);
                Self::ld_8bit_mem(mmu, address, registers.a()) }))
        };
        instructions_8bit[0xEE] = Instruction{ name: String::from("XOR A, d8"), opcode: 0xE, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::xor(registers, registers.a(), value) })) };
        instructions_8bit[0xEF] = Instruction{ name: String::from("RST 5"), opcode: 0xEF, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::call(mmu, registers, 0, 0x28) })) };

        instructions_8bit[0xF0] = Instruction{ name: String::from("LD A, (a8)"), opcode: 0xF0, length: 2, cycles: 3,
            operation: Operation::Unary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, lower_byte: u8| {
                let address = concatenate_bytes(lower_byte, 0xFF) as usize;
                Self::ld_8bit(registers, Register::A, mmu.read_byte(address).unwrap());
            }))
        };
        instructions_8bit[0xF1] = Instruction{ name: String::from("POP AF"), opcode: 0xF1, length: 1, cycles: 3,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::pop(mmu, registers, Register::AF) })) };
        instructions_8bit[0xF2] = Instruction{ name: String::from("LD A, (C)"), opcode: 0xF2, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| {
                let address = concatenate_bytes(registers.c(), 0xFF);
                Self::ld_8bit(registers, Register::A, mmu.read_byte(address as usize).unwrap()) }))
        };
        instructions_8bit[0xF3] = Instruction{ name: String::from("DI"), opcode: 0xF3, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, _| { mmu.set_ime(0) })) };
        instructions_8bit[0xF5] = Instruction{ name: String::from("PUSH AF"), opcode: 0xF5, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::push(mmu, registers, registers.af()) })) };
        instructions_8bit[0xF6] = Instruction{ name: String::from("OR A, d8"), opcode: 0xF6, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::or(registers, registers.a(), value) })) };
        instructions_8bit[0xF7] = Instruction{ name: String::from("RST 6"), opcode: 0xF7, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::call(mmu, registers, 0, 0x30) })) };
        instructions_8bit[0xFA] = Instruction{ name: String::from("LD A, (a16)"), opcode: 0xFA, length: 3, cycles: 4,
            operation: Operation::Binary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers, lower_byte: u8, higher_byte: u8| {
                let address = concatenate_bytes(lower_byte, higher_byte) as usize;
                Self::ld_8bit(registers, Register::A, mmu.read_byte(address).unwrap());
            }))
        };
        instructions_8bit[0xFB] = Instruction{ name: String::from("EI"), opcode: 0xFB, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { mmu.set_ime(1) })) };
        instructions_8bit[0xFE] = Instruction{ name: String::from("CP d8"), opcode: 0xF8, length: 2, cycles: 2,
            operation: Operation::Unary(Rc::new(|_, registers: &mut Registers, value: u8| { Self::cp(registers, registers.a(), value) })) };
        instructions_8bit[0xFF] = Instruction{ name: String::from("RST 7"), opcode: 0xFF, length: 1, cycles: 4,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::call(mmu, registers, 0, 0x38) })) };

        // endregion

        // region 16-bit opcodes

        let mut instructions_16bit: [Instruction; 256] = unsafe { mem::transmute(instructions_16bit) };

        instructions_16bit[0x30] = Instruction{ name: String::from("SWAP B"), opcode: 0x30, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::swap(mmu, registers, |_, r| r.b_ref()) })) };
        instructions_16bit[0x31] = Instruction{ name: String::from("SWAP C"), opcode: 0x31, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::swap(mmu, registers, |_, r| r.c_ref()) })) };
        instructions_16bit[0x32] = Instruction{ name: String::from("SWAP D"), opcode: 0x32, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::swap(mmu, registers,|_, r| r.d_ref()) })) };
        instructions_16bit[0x33] = Instruction{ name: String::from("SWAP E"), opcode: 0x33, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::swap(mmu, registers, |_, r| r.e_ref()) })) };
        instructions_16bit[0x34] = Instruction{ name: String::from("SWAP H"), opcode: 0x34, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::swap(mmu, registers, |_, r| r.h_ref()) })) };
        instructions_16bit[0x35] = Instruction{ name: String::from("SWAP L"), opcode: 0x35, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::swap(mmu, registers, |_, r| r.l_ref()) })) };
        instructions_16bit[0x36] = Instruction{ name: String::from("SWAP (HL)"), opcode: 0x36, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::swap(mmu, registers, |m, r| m.get_byte_ref(r.hl() as usize).unwrap()) })) };
        instructions_16bit[0x37] = Instruction{ name: String::from("SWAP A"), opcode: 0x37, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::srl(mmu, registers, |_, r| r.a_ref()) })) };
        instructions_16bit[0x38] = Instruction{ name: String::from("SRL B"), opcode: 0x38, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::srl(mmu, registers, |m, r| r.b_ref()) })) };
        instructions_16bit[0x39] = Instruction{ name: String::from("SRL C"), opcode: 0x39, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::srl(mmu, registers, |m, r| r.c_ref()) })) };
        instructions_16bit[0x3A] = Instruction{ name: String::from("SRL D"), opcode: 0x3A, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::srl(mmu, registers, |m, r| r.d_ref()) })) };
        instructions_16bit[0x3B] = Instruction{ name: String::from("SRL E"), opcode: 0x3B, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::srl(mmu, registers, |m, r| r.e_ref()) })) };
        instructions_16bit[0x3C] = Instruction{ name: String::from("SRL H"), opcode: 0x3C, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::srl(mmu, registers, |m, r| r.h_ref()) })) };
        instructions_16bit[0x3D] = Instruction{ name: String::from("SRL L"), opcode: 0x3D, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::srl(mmu, registers, |m, r| r.l_ref()) })) };
        instructions_16bit[0x3E] = Instruction{ name: String::from("SRL (HL)"), opcode: 0x3E, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::srl(mmu, registers, |m, r|  m.get_byte_ref(r.hl() as usize).unwrap()) })) };
        instructions_16bit[0x3F] = Instruction{ name: String::from("SRL A"), opcode: 0x3F, length: 2, cycles: 2,
            operation: Operation::Nullary(Rc::new(|mmu: &mut Mmu, registers: &mut Registers| { Self::srl(mmu, registers, |m, r| r.a_ref()) })) };

        // BIT (0x40 - 0x7F)
        for bit in 0..=7 {
            for (i, (destination_name, accessor)) in source_regs.iter().enumerate() {
                let name = format!("BIT {}, {}", bit, destination_name);
                let opcode = 0x40 + bit * 8 + i as u8;
                let cycles = if name == "(HL)" { 3 } else { 2 };
                let accessor = *accessor;
                let operation = if *destination_name == "(HL)" {
                    Operation::Nullary(Rc::new(move |mmu: &mut Mmu, registers: &mut Registers| {
                        let bit = bit;
                        Self::bit(registers, mmu.read_byte(registers.hl() as usize).unwrap(), bit);
                    }))
                } else {
                    Operation::Nullary(Rc::new(move |_, registers: &mut Registers| {
                        let bit = bit;
                        Self::bit(registers, accessor(registers), bit);
                    }))
                };

                instructions_16bit[opcode as usize] = Instruction { name, opcode, length: 2, cycles, operation };
            }
        }

        let destination_regs: [(&str, fn(&mut Registers) -> &mut u8); 8] = [
            ("B", Registers::b_ref),
            ("C", Registers::c_ref),
            ("D", Registers::d_ref),
            ("E", Registers::e_ref),
            ("H", Registers::h_ref),
            ("L", Registers::l_ref),
            ("(HL)", |_| panic!("special case")), // handled separately
            ("A", Registers::a_ref),
        ];

        // RES
        for bit in 0..=7 {
            for (i, (destination_name, accessor)) in destination_regs.iter().enumerate() {
                let name = format!("RES {}, {}", bit, destination_name);
                let opcode = 0x80 + bit * 8 + i as u8;
                let cycles = if name == "(HL)" { 4 } else { 2 };
                let accessor = *accessor;
                let operation = if *destination_name == "(HL)" {
                    Operation::Nullary(Rc::new(move |mmu: &mut Mmu, registers: &mut Registers| {
                        let bit = bit;
                        Self::res(mmu.get_byte_ref(registers.hl() as usize).unwrap(), bit);
                    }))
                } else {
                    Operation::Nullary(Rc::new(move |_, registers: &mut Registers| {
                        let bit = bit;
                        Self::res(accessor(registers), bit);
                    }))
                };

                instructions_16bit[opcode as usize] = Instruction { name, opcode, length: 2, cycles, operation };
            }
        }

        // SET
        for bit in 0..=7 {
            for (i, (destination_name, accessor)) in destination_regs.iter().enumerate() {
                let name = format!("SET {}, {}", bit, destination_name);
                let opcode = 0xC0 + bit * 8 + i as u8;
                let cycles = if name == "(HL)" { 4 } else { 2 };
                let accessor = *accessor;
                let operation = if *destination_name == "(HL)" {
                    Operation::Nullary(Rc::new(move |mmu: &mut Mmu, registers: &mut Registers| {
                        let bit = bit;
                        Self::set(mmu.get_byte_ref(registers.hl() as usize).unwrap(), bit);
                    }))
                } else {
                    Operation::Nullary(Rc::new(move |_, registers: &mut Registers| {
                        let bit = bit;
                        Self::set(accessor(registers), bit);
                    }))
                };

                instructions_16bit[opcode as usize] = Instruction { name, opcode, length: 2, cycles, operation };
            }
        }

        // endregion

        InstructionSet { instructions_8bit, instructions_16bit, mmu }
    }

    pub fn fetch_instruction(&self, opcode: u8) -> Instruction {
        self.instructions_8bit[opcode as usize].clone()
    }

    pub fn fetch_instruction_16bit(&self, opcode: u8) -> Instruction {
        self.instructions_16bit[opcode as usize].clone()
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

    // Increment the content of a memory address by 1
    // Flags: Z 0 8-bit -
    fn inc_mem(registers: &mut Registers, mmu: &mut Mmu, address: usize) {
        let original_value = mmu.read_byte(address).unwrap();
        let new_value = original_value + 1;
        mmu.write_byte(address, new_value).unwrap();

        registers.set_zero_flag(new_value == 0);
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(carry_check_add_8bit(original_value, 1));
    }

    // Decrements the contents of a register pair by 1
    // Flags: - - - -
    fn dec_16bit(registers: &mut Registers, register: Register) {
        let original_value = registers.get_16bit_register(register.clone());
        let new_value = original_value - 1;
        registers.set_16bit_register(register, new_value);
    }

    // Increment the content of a memory address by 1
    // Flags: Z 0 8-bit -
    fn dec_mem(registers: &mut Registers, mmu: &mut Mmu, address: usize) {
        let original_value = mmu.read_byte(address).unwrap();
        let new_value = original_value - 1;
        mmu.write_byte(address, new_value).unwrap();

        registers.set_zero_flag(new_value == 0);
        registers.set_subtraction_flag(true);
        registers.set_half_carry_flag(carry_check_sub_8bit(original_value, 1));
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

    // Add a 16 bit value to a register and store the result in that register
    // Flags: - 0 16-bit 16-bit
    fn add_16bit(registers: &mut Registers, register: Register, value: u16) {
        let left_operand = registers.get_16bit_register(register);
        let sum = left_operand + value;

        registers.set_16bit_register(register, sum);

        registers.set_zero_flag(sum == 0);
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(carry_check_add_16bit(left_operand, value));
        registers.set_carry_flag((left_operand as u32 + value as u32) > 0xFFFF);
    }

    fn add_8bit_signed(registers: &mut Registers, register: Register, value: i8) {
        todo!()
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

    // Load the value at the given address
    // Flags: - - - -
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

    fn jp(registers: &mut Registers, address: u16) {
        registers.set_pc(address);
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

    // Pop from the memory stack the program counter PC value pushed when the subroutine was called, returning control to the source program.
    // Flags: - - - -
    fn ret(mmu: &mut Mmu, registers: &mut Registers) {
        let value_higher = mmu.read_byte(registers.sp() as usize).unwrap();
        registers.increment_sp();
        let value_lower = mmu.read_byte(registers.sp() as usize).unwrap();
        registers.increment_sp();

        registers.set_16bit_register(Register::PC, concatenate_bytes(value_lower, value_higher));
    }


    // Tests the bit b of the 8-bit register r.
    // Flags: !rn 0 1 -
    fn bit(registers: &mut Registers, value: u8, bit_position: u8) {
        let bit_check = value & (1 << bit_position);

        registers.set_zero_flag(bit_check == 0);
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(true);
    }

    // Set the given bit to 0 in value
    // Flags: - - - -
    fn res(value: &mut u8, bit_position: u8) {
        *value &= !(1 << bit_position)
    }

    // Set the given bit to 1 in value
    // Flags: - - - -
    fn set(value: &mut u8, bit_position: u8) {
        *value ^= 1 << bit_position;
    }

    // Take the one's complement (i.e., flip all bits) of the contents of register A.
    // Flags: - 1 1 -
    fn cpl(registers: &mut Registers) {
        let value = registers.a();
        registers.set_a(!value);

        registers.set_subtraction_flag(true);
        registers.set_half_carry_flag(true);
    }

    // Flip the carry flag CY.
    // - 0 0 !CY
    fn ccf(registers: &mut Registers) {
        registers.set_carry_flag(!registers.carry_flag());
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(false);
    }

    // Shift the contents of the lower-order four bits (0-3) of register B to the higher-order four
    // bits (4-7) of the register, and shift the higher-order four bits to the lower-order four bits.
    // Flags: Z 0 0 0
    fn swap<F>(mmu: &mut Mmu, registers: &mut Registers, mut get_value: F) where for<'a> F: FnMut(&'a mut Mmu, &'a mut Registers) -> &'a mut u8 {
        let value = get_value(mmu, registers);

        let higher_bits = (*value & 0xF0) >> 4;
        let lower_bits = *value & 0x0F;
        let new_value = (lower_bits << 4) | higher_bits;

        *value = new_value;

        registers.set_zero_flag(new_value == 0);
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(false);
        registers.set_carry_flag(false);
    }

    // Shift the contents of register B to the right.
    // Flags: Z 0 0 B0
    fn srl<F>(mmu: &mut Mmu, registers: &mut Registers, mut get_value: F) where for<'a> F: FnMut(&'a mut Mmu, &'a mut Registers) -> &'a mut u8 {
        let value = get_value(mmu, registers);

        let lsb = *value & 1;
        let new_value = *value >> 1;

        registers.set_zero_flag(new_value == 0);
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(false);
        registers.set_carry_flag(new_value == 1);
    }
}