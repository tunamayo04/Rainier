mod cpu;
mod mmu;

use std::cell::RefCell;
use anyhow::Result;
use std::path::Path;
use std::rc::Rc;
use cpu::*;
use mmu::*;

struct Rainier {
    mmu: Rc<RefCell<Mmu>>,
    cpu: Cpu,
}

impl Rainier {
    pub fn new() -> Result<Self> {
        let mmu = Rc::new(RefCell::new(Mmu::new(Path::new("roms/tetris.gb"))?));
        let cpu = Cpu::new(mmu.clone());

        Ok(Rainier { cpu, mmu })
    }

    pub fn boot(&self) {
        println!("Booting");
    }

}

fn main() -> Result<()> {
    let rainier = Rainier::new()?;
    rainier.boot();

    Ok(())
}
