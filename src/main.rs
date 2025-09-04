mod cpu;
mod mmu;

use anyhow::Result;
use std::path::Path;
use cpu::*;
use mmu::*;

struct Rainier {
    cpu: Cpu,
    mmu: Mmu,
}

impl Rainier {
    pub fn new() -> Result<Self> {
        let cpu = Cpu::new();
        let mmu = Mmu::new(Path::new("roms/tetris.gb"))?;

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
