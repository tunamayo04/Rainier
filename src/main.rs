mod cpu;
mod mmu;

use std::cell::RefCell;
use anyhow::Result;
use std::path::Path;
use std::rc::Rc;
use cpu::*;
use log::debug;
use mmu::*;

struct Rainier {
    mmu: Rc<RefCell<Mmu>>,
    cpu: Cpu,
}

impl Rainier {
    pub fn new() -> Result<Self> {
        let mmu = Rc::new(RefCell::new(Mmu::new()?));
        let cpu = Cpu::new(mmu.clone());

        Ok(Rainier { cpu, mmu })
    }

    // Set up the system as it would be after running the boot rom
    pub fn boot(&mut self) -> Result<()> {
        let registers = &mut self.cpu.registers;
        registers.set_a(0x01);
        registers.set_b(0xff);
        registers.set_c(0x13);
        registers.set_d(0x00);
        registers.set_e(0xc1);
        registers.set_h(0x84);
        registers.set_l(0x03);
        registers.set_pc(0x0100);
        registers.set_sp(0xfffe);
        registers.clear_all_flags();

        {
            let mut mmu = self.mmu.borrow_mut();
            mmu.set_p1(0xcf);
            mmu.set_sb(0x00);
            mmu.set_sc(0x7e);
            mmu.set_div(0x18);
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
            mmu.set_stat(0x81);
            mmu.set_scy(0x00);
            mmu.set_scx(0x00);
            mmu.set_ly(0x91);
            mmu.set_lyc(0x00);
            mmu.set_dma(0xff);
            mmu.set_bgp(0xfc);
            mmu.set_wy(0x00);
            mmu.set_wx(0x00);
            mmu.set_ie(0x00);

            mmu.load_cartridge(Path::new("roms/tetris.gb"))?;
        }

        self.cpu.emulation_loop()
    }

}

fn main() -> Result<()> {
    let mut rainier = Rainier::new()?;
    rainier.boot()?;

    Ok(())
}
