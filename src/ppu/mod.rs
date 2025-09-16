mod oam;
mod registers;

use std::cell::RefCell;
use std::rc::Rc;
use anyhow::Result;
use crate::cpu::Cpu;
use crate::mmu::{MemoryRegion, Mmu};
use crate::ppu::registers::Register;

#[derive(Copy, Clone, Default, Eq, PartialEq)]
struct OAMEntry {
    y_position: u8,
    x_position: u8,
    tile_index: u8,
    attributes: u8,
}

enum PPUMode {
    HBlank,
    VBlank,
    OAMScan(u8),
    Draw,
}

pub struct Ppu {
    mmu: Rc<RefCell<Mmu>>,
    cpu: Rc<RefCell<Cpu>>,

    sprite_buffer: [Option<OAMEntry>; 10],

    current_mode: PPUMode,
    current_cycles_count: u32,
}

impl Ppu {
    pub fn new(mmu: Rc<RefCell<Mmu>>, cpu: Rc<RefCell<Cpu>>) -> Self {
        Self {
            mmu,
            cpu,
            sprite_buffer: [None; 10],
            current_mode: PPUMode::OAMScan(0),
            current_cycles_count: 0,
        }
    }

    pub fn emulation_loop(&mut self, cycles: u8) -> Result<()> {
        match self.current_mode {
            PPUMode::OAMScan(sprite_id) => {
                self.oam_scan(sprite_id)?;

                self.current_cycles_count += 2;

                if self.current_cycles_count >= 80 {
                    self.current_mode = PPUMode::Draw;
                }
                else {
                    self.current_mode = PPUMode::OAMScan(sprite_id + 1);
                }

                Ok(())
            }
            _ => { todo!() }
        }
    }

    // Fetch a single sprite in OAM and add it to the buffer if it meets the requirements
    // Each call takes 2 cycles
    fn oam_scan(&mut self, sprite_id: u8) -> Result<()> {
        let sprite = self.fetch_oam_entry(sprite_id)?;
        if self.oam_entry_check(&sprite) {
            let i = self.sprite_buffer.iter().position(|x| x.is_none()).unwrap();
            self.sprite_buffer[i] = Some(sprite);
        }

        Ok(())
    }

    fn fetch_oam_entry(&self, id: u8) -> Result<OAMEntry> {
        let mmu = self.mmu.borrow();

        let address = MemoryRegion::SpriteAttributionTable as usize + (4 * id as usize);
        let y_position = mmu.read_byte(address)?;
        let x_position = mmu.read_byte(address + 1)?;
        let tile_index = mmu.read_byte(address + 2)?;
        let attributes = mmu.read_byte(address + 3)?;

        Ok(OAMEntry { y_position, x_position, tile_index, attributes })
    }

    fn oam_entry_check(&self, entry: &OAMEntry) -> bool {
        let mmu = self.mmu.borrow();

        if !self.sprite_buffer.contains(&None) ||
            entry.x_position < 0 ||
            mmu.ly() + 16 < entry.y_position ||
            mmu.ly() + 16 >= entry.y_position + if self.check_register(Register::SpriteSize) { 16 } else { 8 }
        {
            return false;
        }

        true
    }

    fn fetch_tile_number(&self) -> u8 {

    }
}