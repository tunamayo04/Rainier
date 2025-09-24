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
    OAMScan(u8), // The u8 corresponds to the current sprite id that is being retrieved (0-9)
    Draw(DrawStep),
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum DrawStep {
    Fetch {
        x_pos: u8,
        window_line_counter: u8,
        is_window: bool,
    }
}

pub struct Ppu {
    mmu: Rc<RefCell<Mmu>>,
    cpu: Rc<RefCell<Cpu>>,

    sprite_buffer: [Option<OAMEntry>; 10],

    current_mode: PPUMode,
    current_t_cycles_count: u32,
}

impl Ppu {
    pub fn new(mmu: Rc<RefCell<Mmu>>, cpu: Rc<RefCell<Cpu>>) -> Self {
        Self {
            mmu,
            cpu,
            sprite_buffer: [None; 10],
            current_mode: PPUMode::OAMScan(0),
            current_t_cycles_count: 0,
        }
    }

    pub fn emulation_loop(&mut self, t_cycles: u8) -> Result<()> {
        match self.current_mode {
            PPUMode::OAMScan(sprite_id) => {
                self.oam_scan(sprite_id)?;

                // The OAMScan mode takes 80 TCycles
                self.current_t_cycles_count += 2;
                if self.current_t_cycles_count >= 80 {
                    self.current_mode = PPUMode::Draw{ 0: DrawStep::Fetch { x_pos: 0, window_line_counter: 0, is_window: false } };
                }
                else {
                    self.current_mode = PPUMode::OAMScan(sprite_id + 1);
                }

                Ok(())
            },
            PPUMode::Draw(step) => {
                match step {
                    DrawStep::Fetch { x_pos, window_line_counter, is_window } => {
                        let tile_number = self.fetch_tile_number(x_pos, window_line_counter, is_window);

                    }
                }

                Ok(())
            },
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

    fn fetch_tile_number(&self, x_pos: u8, window_line_counter: u8, is_window: bool) -> usize {
        let mmu = self.mmu.borrow();

        let background_tilemap_base = if self.check_register(Register::BGTileMapSelect) { 0x9C00 } else { 0x9800 };
        let mut offset: usize = x_pos as usize;

        // if not window
        if !is_window {
            offset += mmu.scx() as usize / 8;
        }

        offset &= 0x1f;

        // If background
        if !is_window {
            offset += 32 * (((mmu.ly() + mmu.scy()) & 0xFF) / 8) as usize;
        }
        else {
            offset += 32 * (window_line_counter / 8) as usize;
        }

        offset &= 0x3FF;

        offset
    }
}