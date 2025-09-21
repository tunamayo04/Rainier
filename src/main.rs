#![allow(dead_code, unused_variables)]

mod cpu;
mod mmu;
mod bit_utils;
mod ui;
mod ppu;

use std::cell::RefCell;
use std::env;
use std::io::stdout;
use anyhow::Result;
use std::path::Path;
use std::rc::Rc;
use ratatui::crossterm::event::EnableMouseCapture;
use ratatui::crossterm::execute;
use cpu::*;
use mmu::*;
use crate::ppu::Ppu;
use crate::ui::{Action, App};

#[derive(PartialOrd, PartialEq, Copy, Clone)]
enum EmulationMode {
    Debug(u32),
    Normal,
}

struct Rainier {
    mmu: Rc<RefCell<Mmu>>,
    cpu: Rc<RefCell<Cpu>>,
    ppu: Ppu,
}

impl Rainier {
    pub fn new() -> Result<Self> {
        let mmu = Rc::new(RefCell::new(Mmu::new()?));
        let cpu = Rc::new(RefCell::new(Cpu::new(mmu.clone())));
        let ppu = Ppu::new(mmu.clone(), cpu.clone());

        Ok(Rainier { cpu, mmu, ppu })
    }

    // Set up the system as it would be after running the boot rom
    pub fn boot(&mut self, rom: &Path) -> Result<()> {
        let mut cpu = self.cpu.borrow_mut();

        let registers = &mut cpu.registers;
        registers.set_a(0x01);
        registers.set_b(0x00);
        registers.set_c(0x13);
        registers.set_d(0x00);
        registers.set_e(0xd8);
        registers.set_h(0x01);
        registers.set_l(0x4d);
        registers.set_pc(0x0100);
        registers.set_sp(0xFFFE);
        registers.clear_all_flags();
        registers.set_zero_flag(true);
        registers.set_half_carry_flag(true);
        registers.set_carry_flag(true);

        let mut mmu = self.mmu.borrow_mut();
        mmu.set_p1(0xcf);
        mmu.set_sb(0x00);
        mmu.set_sc(0x7e);
        mmu.set_div(0xAB);
        mmu.set_tima(0x00);
        mmu.set_tma(0x00);
        mmu.set_tac(0xf8);
        mmu.set_iflag(0xe1);
        mmu.set_nr10(0x80);
        mmu.set_nr11(0xbf);
        mmu.set_nr12(0xf3);
        mmu.set_nr13(0xff);
        mmu.set_nr14(0xbf);
        mmu.set_nr21(0x3f);
        mmu.set_nr22(0x00);
        mmu.set_nr23(0xff);
        mmu.set_nr24(0xbf);
        mmu.set_nr30(0x7f);
        mmu.set_nr31(0xff);
        mmu.set_nr32(0x9f);
        mmu.set_nr33(0xff);
        mmu.set_nr34(0xbf);
        mmu.set_nr41(0xff);
        mmu.set_nr42(0x00);
        mmu.set_nr43(0x00);
        mmu.set_nr44(0xbf);
        mmu.set_nr50(0x77);
        mmu.set_nr51(0xf3);
        mmu.set_nr52(0xf1);
        mmu.set_lcdc(0x91);
        mmu.set_stat(0x85);
        mmu.set_scy(0x00);
        mmu.set_scx(0x00);
        mmu.set_ly(0x00);
        mmu.set_lyc(0x00);
        mmu.set_dma(0xff);
        mmu.set_bgp(0xfc);
        mmu.set_wy(0x00);
        mmu.set_wx(0x00);
        mmu.set_ie(0x00);

        mmu.write_byte(0xFF44, 0)?;

        mmu.load_cartridge(rom)
    }
}

fn main() -> Result<()> {
    let rainier = Rc::new(RefCell::new(Rainier::new()?));
    rainier.borrow_mut().boot(Path::new("roms/cpu_instrs/individual/08-misc instrs.gb"))?;

    let mut terminal = ratatui::init();
    let mut debugger = App::new(rainier.clone());

    let emulation_mode = match env::var("mode").unwrap_or(String::from("debug")).as_str() {
        "normal" => EmulationMode::Normal,
        _ => EmulationMode::Debug(1),
    };

    if emulation_mode == EmulationMode::Normal {
        loop {
            rainier.borrow_mut().cpu.borrow_mut().emulation_loop()?;
            //rainier.borrow_mut().ppu.emulation_loop(cycles)?;
        }
    }
    else {
        execute!(stdout(), EnableMouseCapture)?;

        while !debugger.exit {
            debugger.run(&mut terminal)?;

            let mut rainier = rainier.borrow_mut();
            let mut cpu = rainier.cpu.borrow_mut();

            if let Some(requested_action) = &debugger.requested_action {
                match requested_action {
                    Action::Trace | Action::StepOver => {
                        cpu.emulation_loop()?;
                        debugger.requested_action = None;
                        debugger.last_hit_breakpoint = None;
                    }
                    Action::Run => {
                        while !debugger.breakpoints.contains(&cpu.registers.pc()) || debugger.last_hit_breakpoint.map_or(false, |breakpoint| cpu.registers.pc() == breakpoint) {
                            cpu.emulation_loop()?;
                            debugger.last_hit_breakpoint = None;
                        }

                        debugger.requested_action = None;
                        debugger.last_hit_breakpoint = Some(cpu.registers.pc());
                    }
                }
            }
        }

        ratatui::restore();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_test_rom(rom: &str) {
        let rainier = Rc::new(RefCell::new(Rainier::new().unwrap()));
        rainier.borrow_mut().boot(Path::new(rom)).unwrap();

        loop {
            let cycles = rainier.borrow_mut().cpu.borrow_mut().emulation_loop().unwrap();
            //rainier.borrow_mut().ppu.emulation_loop(cycles)?;

            let serial_log = rainier.borrow().cpu.borrow().serial_log.clone();
            if serial_log.contains("Failed") {
                println!("{}", serial_log.trim());
                assert!(false);
                break;
            } else if serial_log.contains("Passed") {
                println!("{}", serial_log.trim());
                assert!(true);
                break;
            }
        }
    }

    #[test]
    fn cpu_instrs_01() {
        run_test_rom("roms/cpu_instrs/individual/01-special.gb");
    }

    #[test]
    fn cpu_instrs_02() {
        run_test_rom("roms/cpu_instrs/individual/02-interrupts.gb");
    }

    #[test]
    fn cpu_instrs_03() {
        run_test_rom("roms/cpu_instrs/individual/03-op sp,hl.gb");
    }

    #[test]
    fn cpu_instrs_04() {
        run_test_rom("roms/cpu_instrs/individual/04-op r,imm.gb");
    }

    #[test]
    fn cpu_instrs_05() {
        run_test_rom("roms/cpu_instrs/individual/05-op rp.gb");
    }

    #[test]
    fn cpu_instrs_06() {
        run_test_rom("roms/cpu_instrs/individual/06-ld r,r.gb");
    }

    #[test]
    fn cpu_instrs_07() {
        run_test_rom("roms/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb");
    }

    #[test]
    fn cpu_instrs_08() {
        run_test_rom("roms/cpu_instrs/individual/08-misc instrs.gb");
    }

    #[test]
    fn cpu_instrs_09() {
        run_test_rom("roms/cpu_instrs/individual/09-op r,r.gb");
    }

    #[test]
    fn cpu_instrs_10() {
        run_test_rom("roms/cpu_instrs/individual/10-bit ops.gb");
    }

    #[test]
    fn cpu_instrs_11() {
        run_test_rom("roms/cpu_instrs/individual/11-op a,(hl).gb");
    }
}