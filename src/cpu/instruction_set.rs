use std::cell::{RefCell};
use std::{fmt, mem};
use std::mem::MaybeUninit;
use std::rc::Rc;
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
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::BC ) })) };
        instructions[0x04] = Instruction{ name: "INC B", opcode: 0x04, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::B) })) };
        instructions[0x05] = Instruction{ name: "DEC B", opcode: 0x05, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::B) })) };
        instructions[0x0C] = Instruction{ name: "INC C", opcode: 0x0C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::C) })) };
        instructions[0x0D] = Instruction{ name: "DEC C", opcode: 0x0D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::C) })) };

        instructions[0x13] = Instruction{ name: "INC DE", opcode: 0x13, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::DE ) })) };
        instructions[0x14] = Instruction{ name: "INC D", opcode: 0x14, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::D) })) };
        instructions[0x15] = Instruction{ name: "DEC D", opcode: 0x15, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::D) })) };
        instructions[0x1C] = Instruction{ name: "INC E", opcode: 0x1C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::E) })) };
        instructions[0x1D] = Instruction{ name: "DEC E", opcode: 0x1D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::E) })) };

        instructions[0x23] = Instruction{ name: "INC HL", opcode: 0x23, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::HL ) })) };
        instructions[0x24] = Instruction{ name: "INC H", opcode: 0x24, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::H) })) };
        instructions[0x25] = Instruction{ name: "DEC H", opcode: 0x25, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::H) })) };
        instructions[0x2C] = Instruction{ name: "INC L", opcode: 0x2C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::L) })) };
        instructions[0x2D] = Instruction{ name: "DEC L", opcode: 0x2D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::L) })) };

        instructions[0x33] = Instruction{ name: "INC SP", opcode: 0x33, length: 1, cycles: 2,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_16bit(registers, Register::SP ) })) };
        instructions[0x3C] = Instruction{ name: "INC A", opcode: 0x3C, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::inc_8bit(registers, Register::A) })) };
        instructions[0x3D] = Instruction{ name: "DEC A", opcode: 0x3D, length: 1, cycles: 1,
            operation: Operation::Nullary(Rc::new(|_, registers: &mut Registers| { Self::dec_8bit(registers, Register::A) })) };

        instructions[0xC3] = Instruction{ name: "JP a16", opcode: 0xC3, length: 3, cycles: 2,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers, lower_bits: u8, upper_bits: u8| { Self::jmp(registers, lower_bits, upper_bits) })) };

        instructions[0x80] = Instruction{ name: "ADD A, B", opcode: 0x80, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.b()) })) };
        instructions[0x81] = Instruction{ name: "ADD A, C", opcode: 0x81, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.c()) })) };
        instructions[0x82] = Instruction{ name: "ADD A, D", opcode: 0x82, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.d()) })) };
        instructions[0x83] = Instruction{ name: "ADD A, E", opcode: 0x83, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(),  registers.e()) })) };
        instructions[0x84] = Instruction{ name: "ADD A, H", opcode: 0x84, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.h()) })) };
        instructions[0x85] = Instruction{ name: "ADD A, L", opcode: 0x85, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.l()) })) };
        instructions[0x86] = Instruction{ name: "ADD A, (HL)", opcode: 0x86, length: 1, cycles: 2,
            operation: Operation::Binary(Rc::new(|mmu: &Mmu, registers: &mut Registers| { Self::add(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x87] = Instruction{ name: "ADD A, A", opcode: 0x87, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::add(registers, registers.a(), registers.a()) })) };
        instructions[0x88] = Instruction{ name: "ADC A, B", opcode: 0x88, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.b()) })) };
        instructions[0x89] = Instruction{ name: "ADC A, C", opcode: 0x89, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.c()) })) };
        instructions[0x8A] = Instruction{ name: "ADC A, D", opcode: 0x8A, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.d()) })) };
        instructions[0x8B] = Instruction{ name: "ADC A, E", opcode: 0x8B, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(),  registers.e()) })) };
        instructions[0x8C] = Instruction{ name: "ADC A, H", opcode: 0x8C, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.h()) })) };
        instructions[0x8D] = Instruction{ name: "ADC A, L", opcode: 0x8D, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.l()) })) };
        instructions[0x8E] = Instruction{ name: "ADC A, (HL)", opcode: 0x8E, length: 1, cycles: 2,
            operation: Operation::Binary(Rc::new(|mmu: &Mmu, registers: &mut Registers| { Self::adc(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x8F] = Instruction{ name: "ADC A, A", opcode: 0x8F, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::adc(registers, registers.a(), registers.a()) })) };

        instructions[0x90] = Instruction{ name: "SUB B", opcode: 0x90, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.b()) })) };
        instructions[0x91] = Instruction{ name: "SUB C", opcode: 0x91, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.c()) })) };
        instructions[0x92] = Instruction{ name: "SUB D", opcode: 0x92, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.d()) })) };
        instructions[0x93] = Instruction{ name: "SUB E", opcode: 0x93, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(),  registers.e()) })) };
        instructions[0x94] = Instruction{ name: "SUB H", opcode: 0x94, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.h()) })) };
        instructions[0x95] = Instruction{ name: "SUB L", opcode: 0x95, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.l()) })) };
        instructions[0x96] = Instruction{ name: "SUB (HL)", opcode: 0x96, length: 1, cycles: 2,
            operation: Operation::Binary(Rc::new(|mmu: &Mmu, registers: &mut Registers| { Self::sub(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x97] = Instruction{ name: "SUB A", opcode: 0x97, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sub(registers, registers.a(), registers.a()) })) };
        instructions[0x98] = Instruction{ name: "SBC B", opcode: 0x98, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.b()) })) };
        instructions[0x99] = Instruction{ name: "SBC C", opcode: 0x99, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.c()) })) };
        instructions[0x9A] = Instruction{ name: "SBC D", opcode: 0x9A, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.d()) })) };
        instructions[0x9B] = Instruction{ name: "SBC E", opcode: 0x9B, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(),  registers.e()) })) };
        instructions[0x9C] = Instruction{ name: "SBC H", opcode: 0x9C, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.h()) })) };
        instructions[0x9D] = Instruction{ name: "SBC L", opcode: 0x9D, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.l()) })) };
        instructions[0x9E] = Instruction{ name: "SBC (HL)", opcode: 0x9E, length: 1, cycles: 2,
            operation: Operation::Binary(Rc::new(|mmu: &Mmu, registers: &mut Registers| { Self::sbc(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0x9F] = Instruction{ name: "SBC A", opcode: 0x9F, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::sbc(registers, registers.a(), registers.a()) })) };

        instructions[0xA0] = Instruction{ name: "AND B", opcode: 0xA0, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.b()) })) };
        instructions[0xA1] = Instruction{ name: "AND C", opcode: 0xA1, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.c()) })) };
        instructions[0xA2] = Instruction{ name: "AND D", opcode: 0xA2, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.d()) })) };
        instructions[0xA3] = Instruction{ name: "AND E", opcode: 0xA3, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(),  registers.e()) })) };
        instructions[0xA4] = Instruction{ name: "AND H", opcode: 0xA4, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.h()) })) };
        instructions[0xA5] = Instruction{ name: "AND L", opcode: 0xA5, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::and(registers, registers.a(), registers.l()) })) };
        instructions[0xA6] = Instruction{ name: "AND (HL)", opcode: 0xA6, length: 1, cycles: 2,
            operation: Operation::Binary(Rc::new(|mmu: &Mmu, registers: &mut Registers| { Self::and(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0xA7] = Instruction{ name: "AND A", opcode: 0xA7, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.a()) })) };
        instructions[0xA8] = Instruction{ name: "XOR B", opcode: 0xA8, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.b()) })) };
        instructions[0xA9] = Instruction{ name: "XOR C", opcode: 0xA9, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.c()) })) };
        instructions[0xAA] = Instruction{ name: "XOR D", opcode: 0xAA, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.d()) })) };
        instructions[0xAB] = Instruction{ name: "XOR E", opcode: 0xAB, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(),  registers.e()) })) };
        instructions[0xAC] = Instruction{ name: "XOR H", opcode: 0xAC, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.h()) })) };
        instructions[0xAD] = Instruction{ name: "XOR L", opcode: 0xAD, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.l()) })) };
        instructions[0xAE] = Instruction{ name: "XOR (HL)", opcode: 0xAE, length: 1, cycles: 2,
            operation: Operation::Binary(Rc::new(|mmu: &Mmu, registers: &mut Registers| { Self::xor(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0xAF] = Instruction{ name: "XOR A", opcode: 0xAF, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::xor(registers, registers.a(), registers.a()) })) };

        instructions[0xB0] = Instruction{ name: "OR B", opcode: 0xB0, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.b()) })) };
        instructions[0xB1] = Instruction{ name: "OR C", opcode: 0xB1, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.c()) })) };
        instructions[0xB2] = Instruction{ name: "OR D", opcode: 0xB2, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.d()) })) };
        instructions[0xB3] = Instruction{ name: "OR E", opcode: 0xB3, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(),  registers.e()) })) };
        instructions[0xB4] = Instruction{ name: "OR H", opcode: 0xB4, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.h()) })) };
        instructions[0xB5] = Instruction{ name: "OR L", opcode: 0xB5, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.l()) })) };
        instructions[0xB6] = Instruction{ name: "OR (HL)", opcode: 0xB6, length: 1, cycles: 2,
            operation: Operation::Binary(Rc::new(|mmu: &Mmu, registers: &mut Registers| { Self::or(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0xB7] = Instruction{ name: "OR A", opcode: 0xB7, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::or(registers, registers.a(), registers.a()) })) };
        instructions[0xB8] = Instruction{ name: "CP B", opcode: 0xB8, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.b()) })) };
        instructions[0xB9] = Instruction{ name: "CP C", opcode: 0xB9, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.c()) })) };
        instructions[0xBA] = Instruction{ name: "CP D", opcode: 0xBA, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.d()) })) };
        instructions[0xBB] = Instruction{ name: "CP E", opcode: 0xBB, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(),  registers.e()) })) };
        instructions[0xBC] = Instruction{ name: "CP H", opcode: 0xBC, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.h()) })) };
        instructions[0xBD] = Instruction{ name: "CP L", opcode: 0xBD, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.l()) })) };
        instructions[0xBE] = Instruction{ name: "CP (HL)", opcode: 0xBE, length: 1, cycles: 2,
            operation: Operation::Binary(Rc::new(|mmu: &Mmu, registers: &mut Registers| { Self::cp(registers, registers.a(), mmu.read_byte(registers.hl() as usize).unwrap()) })) };
        instructions[0xBF] = Instruction{ name: "CP A", opcode: 0xBF, length: 1, cycles: 1,
            operation: Operation::Binary(Rc::new(|_, registers: &mut Registers| { Self::cp(registers, registers.a(), registers.a()) })) };

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
        registers.set_half_carry_flag(Self::carry_check_add_8bit(original_value, 1));
    }

    // Decrement the contents of a register by 1
    // Flags: Z 1 8-bit -
    fn dec_8bit(registers: &mut Registers, register: Register) {
        let original_value = registers.get_8bit_register(register.clone());
        let new_value = original_value - 1;
        registers.set_8bit_register(register, new_value);

        registers.set_zero_flag(new_value == 0);
        registers.set_subtraction_flag(true);
        registers.set_half_carry_flag(Self::carry_check_sub_8bit(original_value, 1));
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
        let address = lower_order_byte as u16 | ((higher_order_byte as u16) << 8);
        registers.set_pc(address);
    }

    // Add two values and store the results in register A
    // Flags: Z 0 8-bit 8-bit
    fn add(registers: &mut Registers, left_operator: u8, right_operator: u8) {
        let sum = left_operator + right_operator;

        registers.set_8bit_register(Register::A, sum);

        registers.set_zero_flag(sum == 0);
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(Self::carry_check_add_8bit(left_operator, right_operator));
        registers.set_carry_flag((left_operator as u16 + right_operator as u16) > 0xFF);
    }

    // Add two values along with the carry flag and store the content in register A
    // Flags: Z 0 8-bit 8-bit
    fn adc(registers: &mut Registers, left_operator: u8, right_operator: u8) {
        let sum = left_operator + right_operator + registers.carry_flag() as u8;

        registers.set_8bit_register(Register::A, sum);

        registers.set_zero_flag(sum == 0);
        registers.set_subtraction_flag(false);
        registers.set_half_carry_flag(Self::carry_check_add_8bit(left_operator, right_operator));
        registers.set_carry_flag((left_operator as u16 + right_operator as u16) > 0xFF);
    }

    // Subtract two values and store the result in register A
    // Flags: Z 1 8-bit 8-bit
    fn sub(registers: &mut Registers, left_operator: u8, right_operator: u8) {
        let difference = left_operator - right_operator;

        registers.set_8bit_register(Register::A, difference);

        registers.set_zero_flag(difference == 0);
        registers.set_subtraction_flag(true);
        registers.set_half_carry_flag(Self::carry_check_sub_8bit(left_operator, right_operator));
        registers.set_carry_flag((left_operator as i16 - right_operator as i16) < 0);
    }

    // Subtract two values and the carry flag and store the result in register A
    // Flags: Z 1 8-bit 8-bit
    fn sbc(registers: &mut Registers, left_operator: u8, right_operator: u8) {
        let difference = left_operator - right_operator - registers.carry_flag() as u8;

        registers.set_8bit_register(Register::A, difference);

        registers.set_zero_flag(difference == 0);
        registers.set_subtraction_flag(true);
        registers.set_half_carry_flag(Self::carry_check_sub_8bit(left_operator, right_operator));
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
        registers.set_half_carry_flag(Self::carry_check_sub_8bit(left_operator, right_operator));
        registers.set_carry_flag((left_operator as i16 - right_operator as i16) < 0);
    }

    fn carry_check_add_8bit(a: u8, b: u8) -> bool {
        (((a & 0xF) + (b & 0xF)) & 0x10) == 0x10
    }

    fn carry_check_sub_8bit(a: u8, b: u8) -> bool {
        (((a & 0xF) - (b & 0xF)) & 0x10) == 0x10
    }
}