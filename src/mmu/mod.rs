/*
                                    -- Memory map --
Start	End	    Description	Notes
0000	3FFF	16 KiB ROM bank 00	From cartridge, usually a fixed bank
4000	7FFF	16 KiB ROM Bank 01–NN	From cartridge, switchable bank via mapper (if any)
8000	9FFF	8 KiB Video RAM (VRAM)	In CGB mode, switchable bank 0/1
A000	BFFF	8 KiB External RAM	From cartridge, switchable bank if any
C000	CFFF	4 KiB Work RAM (WRAM)
D000	DFFF	4 KiB Work RAM (WRAM)	In CGB mode, switchable bank 1–7
E000	FDFF	Echo RAM (mirror of C000–DDFF)	Nintendo says use of this area is prohibited.
FE00	FE9F	Object attribute memory (OAM)
FEA0	FEFF	Not Usable	Nintendo says use of this area is prohibited.
FF00	FF7F	I/O Registers
FF80	FFFE	High RAM (HRAM)
FFFF	FFFF	Interrupt Enable register (IE)
*/
mod io;

use std::{fs, path};
use anyhow::{Context, Result};
use crate::mmu::MemoryRegion::*;

const MEMORY_BANK_SIZE: usize = 0xFFFF;
const ROM_BANK_SIZE: usize = 0x4000;
const VIDEO_RAM_SIZE: usize = 0x2000;
const EXTERNAL_RAM_SIZE: usize = 0x2000;
const WORK_RAM_SIZE: usize = 0x2000;
const SPRITE_ATTRIBUTION_TABLE_SIZE: usize = 0x100;
const IO_SIZE: usize = 0x80;
const HIGH_RAM_SIZE: usize = 0x80;

enum MemoryRegion {
    RomBankZero = 0x0000,
    RomBankSwap = 0x4000,
    VideoRam = 0x8000,
    ExternalRam = 0xA000,
    WorkRam = 0xC000,
    EchoRam = 0xE000,
    SpriteAttributionTable = 0xFE00,
    IO = 0xFF00,
    HighRam = 0xFF80,
    InterruptEnableRegister = 0xFFFF,
}

impl MemoryRegion {
    pub fn from_address(address: usize) -> Result<Self> {
        match address {
            0x0000..=0x3FFF => Ok(RomBankZero),
            0x4000..=0x7FFF => Ok(RomBankSwap),
            0x8000..=0x9FFF => Ok(VideoRam),
            0xA000..=0xBFFF => Ok(ExternalRam),
            0xC000..=0xDFFF => Ok(WorkRam),
            0xE000..=0xFDFF => Ok(EchoRam),
            0xFE00..=0xFE9F => Ok(SpriteAttributionTable),
            0xFF00..=0xFF7F => Ok(IO),
            0xFF80..=0xFFFE => Ok(HighRam),
            0xFFFF => Ok(InterruptEnableRegister),
            _ => Err(anyhow::anyhow!("Illegal address"))
        }
    }
}

pub struct Mmu {
    rom_bank_zero: [u8; ROM_BANK_SIZE],
    rom_bank_swap: [u8; ROM_BANK_SIZE],

    video_ram: [u8; VIDEO_RAM_SIZE],
    external_ram: [u8; EXTERNAL_RAM_SIZE],
    work_ram: [u8; WORK_RAM_SIZE],
    echo_ram: [u8; WORK_RAM_SIZE],

    sprite_attribution_table: [u8; SPRITE_ATTRIBUTION_TABLE_SIZE],
    io: [u8; IO_SIZE],
    high_ram: [u8; HIGH_RAM_SIZE],

    interrupt_enable_register: u8,

    cartridge_data: Vec<u8>,
}

impl Mmu {
    pub fn new() -> Result<Self> {
        Ok(Self {
            rom_bank_zero: [0; ROM_BANK_SIZE],
            rom_bank_swap: [0; ROM_BANK_SIZE],

            video_ram: [0; VIDEO_RAM_SIZE],
            external_ram: [0; EXTERNAL_RAM_SIZE],
            work_ram: [0; WORK_RAM_SIZE],
            echo_ram: [0; WORK_RAM_SIZE],

            sprite_attribution_table: [0; SPRITE_ATTRIBUTION_TABLE_SIZE],
            io: [0; IO_SIZE],
            high_ram: [0; HIGH_RAM_SIZE],

            interrupt_enable_register: 0,

            cartridge_data: Vec::new(),
        })
    }

    pub fn load_cartridge(&mut self, path: &path::Path) -> Result<()> {
        let data: Vec<u8> = fs::read(path).context("Failed to read ROM")?;

        self.cartridge_data = data;
        self.load_rom_bank(0);

        Ok(())
    }

    fn load_rom_bank(&mut self, bank_id: usize) {
        let start_address = bank_id * ROM_BANK_SIZE;
        let end_address = start_address + ROM_BANK_SIZE;

        // todo: bounds check?
        let rom_bank = &self.cartridge_data[start_address..end_address];

        match bank_id {
            0 => {
                self.rom_bank_zero.clone_from_slice(rom_bank);
            },
            _ => {
                self.rom_bank_swap.clone_from_slice(rom_bank);
            }
        }
    }

    fn load_rom(path: &path::Path) -> Result<[u8; ROM_BANK_SIZE]> {
        let data: Vec<u8> = fs::read(path).context("Failed to read ROM")?;

        println!("{}", data.len());

        data.try_into()
            .map_err(|v: Vec<u8>| anyhow::anyhow!("Expected ROM size 0x8000, got {:#x}", v.len()))
    }

    pub fn read_byte(&self, address: usize) -> Result<u8> {
        Ok(match MemoryRegion::from_address(address)? {
            RomBankZero => {
                let relative_address = address - RomBankZero as usize;
                self.rom_bank_zero[relative_address]
            }
            RomBankSwap => {
                let relative_address = address - RomBankSwap as usize;
                self.rom_bank_swap[relative_address]
            }
            VideoRam => {
                let relative_address = address - VideoRam as usize;
                self.video_ram[relative_address]
            }
            ExternalRam => {
                let relative_address = address - ExternalRam as usize;
                self.external_ram[relative_address]
            }
            WorkRam => {
                let relative_address = address - WorkRam as usize;
                self.work_ram[relative_address]
            }
            EchoRam => {
                let relative_address = address - EchoRam as usize;
                self.echo_ram[relative_address]
            }
            SpriteAttributionTable => {
                let relative_address = address - SpriteAttributionTable as usize;
                self.sprite_attribution_table[relative_address]
            }
            IO => {
                let relative_address = address - IO as usize;
                self.io[relative_address]
            }
            HighRam => {
                let relative_address = address - HighRam as usize;
                self.high_ram[relative_address]
            }
            InterruptEnableRegister => {
                self.interrupt_enable_register
            }
        })
    }

    pub fn write_byte(&mut self, address: usize, value: u8) -> Result<()> {
        match MemoryRegion::from_address(address)? {
            RomBankZero | RomBankSwap => {
                Err(anyhow::anyhow!("Attempted to write into illegal memory region"))
            }
            VideoRam => {
                let relative_address = address - VideoRam as usize;
                self.video_ram[relative_address] = value;

                Ok(())
            }
            ExternalRam => {
                let relative_address = address - ExternalRam as usize;
                self.external_ram[relative_address] = value;

                Ok(())
            }
            WorkRam => {
                let relative_address = address - WorkRam as usize;
                self.work_ram[relative_address] = value;

                Ok(())
            }
            EchoRam => {
                Err(anyhow::anyhow!("Attempted to write into illegal memory region"))
            }
            SpriteAttributionTable => {
                let relative_address = address - SpriteAttributionTable as usize;
                self.sprite_attribution_table[relative_address] = value;

                Ok(())
            }
            IO => {
                let relative_address = address - IO as usize;
                self.io[relative_address] = value;

                Ok(())
            }
            HighRam => {
                let relative_address = address - HighRam as usize;
                self.high_ram[relative_address] = value;

                Ok(())
            }
            InterruptEnableRegister => {
                self.interrupt_enable_register = value;

                Ok(())
            }
        }
    }
}