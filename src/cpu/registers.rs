#[derive(Debug, Copy, Clone)]
pub enum Register {
    A, B, C, D, E, H, L, AF, BC, DE, HL, SP, PC,
}

#[derive(Debug, Default, Clone)]
pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

#[repr(u8)]
pub enum Flag {
    Zero = 1 << 7,
    Subtraction = 1 << 6,
    HalfCarry = 1 << 5,
    Carry = 1 << 4,
}

impl Registers {
    pub fn new() -> Self {
        Registers::default()
    }

    pub fn get_8bit_register(&self, register: Register) -> u8 {
        match register {
            Register::A => self.a(),
            Register::B => self.b(),
            Register::C => self.c(),
            Register::D => self.d(),
            Register::E => self.e(),
            Register::H => self.h(),
            Register::L => self.l(),
            _ => panic!("Not an 8-bit register"),
        }
    }

    pub fn set_8bit_register(&mut self, register: Register, value: u8) {
        match register {
            Register::A => self.set_a(value),
            Register::B => self.set_b(value),
            Register::C => self.set_c(value),
            Register::D => self.set_d(value),
            Register::E => self.set_e(value),
            Register::H => self.set_h(value),
            Register::L => self.set_l(value),
            _ => panic!("Not an 8-bit register"),
        }
    }

    pub fn get_16bit_register(&self, register: Register) -> u16 {
        match register {
            Register::AF => self.af(),
            Register::BC => self.bc(),
            Register::DE => self.de(),
            Register::HL => self.hl(),
            Register::SP => self.sp(),
            Register::PC => self.pc(),
            _ => panic!("Not a 16-bit register"),
        }
    }

    pub fn set_16bit_register(&mut self, register: Register, value: u16) {
        match register {
            Register::AF => self.set_af(value),
            Register::BC => self.set_bc(value),
            Register::DE => self.set_de(value),
            Register::HL => self.set_hl(value),
            Register::SP => self.set_sp(value),
            Register::PC => self.set_pc(value),
            _ => panic!("Not an 16-bit register"),
        }
    }

    // --- 8-bit register getters/setters ---
    pub fn a(&self) -> u8 { self.a }
    pub fn a_ref(&mut self) -> &mut u8 { &mut self.a }
    pub fn set_a(&mut self, val: u8) { self.a = val }

    pub fn b(&self) -> u8 { self.b }
    pub fn b_ref(&mut self) -> &mut u8 { &mut self.b }
    pub fn set_b(&mut self, val: u8) { self.b = val }

    pub fn c(&self) -> u8 { self.c }
    pub fn c_ref(&mut self) -> &mut u8 { &mut self.c }
    pub fn set_c(&mut self, val: u8) { self.c = val }

    pub fn d(&self) -> u8 { self.d }
    pub fn d_ref(&mut self) -> &mut u8 { &mut self.d }
    pub fn set_d(&mut self, val: u8) { self.d = val }

    pub fn e(&self) -> u8 { self.e }
    pub fn e_ref(&mut self) -> &mut u8 { &mut self.e }
    pub fn set_e(&mut self, val: u8) { self.e = val }

    pub fn f(&self) -> u8 { self.f }
    pub fn f_ref(&mut self) -> &mut u8 { &mut self.f }
    pub fn set_f(&mut self, val: u8) { self.f = val & 0xF0 } // lower 4 bits always 0

    pub fn h(&self) -> u8 { self.h }
    pub fn h_ref(&mut self) -> &mut u8 { &mut self.h }
    pub fn set_h(&mut self, val: u8) { self.h = val }

    pub fn l(&self) -> u8 { self.l }
    pub fn l_ref(&mut self) -> &mut u8 { &mut self.l }
    pub fn set_l(&mut self, val: u8) { self.l = val }

    // --- 16-bit register pair getters/setters ---
    pub fn af(&self) -> u16 { ((self.a as u16) << 8) | self.f as u16 }
    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val & 0xF0) as u8; // lower 4 bits of F always 0
    }

    pub fn bc(&self) -> u16 { ((self.b as u16) << 8) | self.c as u16 }
    pub fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = val as u8;
    }

    pub fn de(&self) -> u16 { ((self.d as u16) << 8) | self.e as u16 }
    pub fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = val as u8;
    }

    pub fn hl(&self) -> u16 { ((self.h as u16) << 8) | self.l as u16 }
    pub fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = val as u8;
    }

    // --- SP and PC ---
    pub fn sp(&self) -> u16 { self.sp }
    pub fn set_sp(&mut self, val: u16) { self.sp = val }
    pub fn increment_sp(&mut self) { self.sp += 1 }
    pub fn decrement_sp(&mut self) { self.sp -= 1 }

    pub fn pc(&self) -> u16 { self.pc }
    pub fn set_pc(&mut self, val: u16) { self.pc = val }
    pub fn increment_pc(&mut self) { self.pc += 1 }

    // --- Flag getters/setters ---
    pub fn get_flag(&self, flag: Flag) -> bool { (self.f & flag as u8) != 0 }
    pub fn set_flag(&mut self, flag: Flag, val: bool) {
        if val {
            self.f |= flag as u8;
        } else {
            self.f &= !(flag as u8);
        }
    }
    pub fn clear_flag(&mut self, flag: Flag) {
        self.f &= !(flag as u8);
    }
    pub fn flip_flag(&mut self, flag: Flag) {
        self.f ^= flag as u8;
    }
    pub fn clear_all_flags(&mut self) {
        self.f = 0;
    }

    pub fn zero_flag(&self) -> bool { self.get_flag(Flag::Zero) }
    pub fn set_zero_flag(&mut self, val: bool) { self.set_flag(Flag::Zero, val) }

    pub fn subtraction_flag(&self) -> bool { self.get_flag(Flag::Subtraction) }
    pub fn set_subtraction_flag(&mut self, val: bool) { self.set_flag(Flag::Subtraction, val) }

    pub fn half_carry_flag(&self) -> bool { self.get_flag(Flag::HalfCarry) }
    pub fn set_half_carry_flag(&mut self, val: bool) { self.set_flag(Flag::HalfCarry, val) }

    pub fn carry_flag(&self) -> bool { self.get_flag(Flag::Carry) }
    pub fn set_carry_flag(&mut self, val: bool) { self.set_flag(Flag::Carry, val) }
}