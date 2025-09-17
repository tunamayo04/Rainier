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
const ECHO_RAM_SIZE: usize = 0x1E00;
const SPRITE_ATTRIBUTION_TABLE_SIZE: usize = 0xA0;
const UNUSABLE_MEMORY_SIZE: usize = 0x60;
const IO_SIZE: usize = 0x80;
const HIGH_RAM_SIZE: usize = 0x7f;

pub enum MemoryRegion {
    RomBankZero = 0x0000,
    RomBankSwap = 0x4000,
    VideoRam = 0x8000,
    ExternalRam = 0xA000,
    WorkRam = 0xC000,
    EchoRam = 0xE000,
    SpriteAttributionTable = 0xFE00,
    Unusable = 0xFEA0,
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
            0xFEA0..=0xFEFF => Ok(Unusable),
            0xFF00..=0xFF7F => Ok(IO),
            0xFF80..=0xFFFE => Ok(HighRam),
            0xFFFF => Ok(InterruptEnableRegister),
            add => Err(anyhow::anyhow!("Illegal address {:X}", add))
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            RomBankZero => String::from("ROM0"),
            RomBankSwap => String::from("ROM1"),
            VideoRam => String::from("VRA0"),
            ExternalRam => String::from("SRA0"),
            WorkRam => String::from("WRA0"),
            EchoRam => String::from("ECHO"),
            SpriteAttributionTable => String::from("OAM"),
            Unusable => String::from("----"),
            IO => String::from("I/O "),
            HighRam => String::from("HRAM"),
            InterruptEnableRegister => String::from("IER ")
        }
    }
}

pub struct Mmu {
    rom_bank_zero: [u8; ROM_BANK_SIZE],
    rom_bank_swap: [u8; ROM_BANK_SIZE],

    video_ram: [u8; VIDEO_RAM_SIZE],
    external_ram: [u8; EXTERNAL_RAM_SIZE],
    work_ram: [u8; WORK_RAM_SIZE],
    echo_ram: [u8; ECHO_RAM_SIZE],

    sprite_attribution_table: [u8; SPRITE_ATTRIBUTION_TABLE_SIZE],
    unusable: [u8; UNUSABLE_MEMORY_SIZE],
    io: [u8; IO_SIZE],
    high_ram: [u8; HIGH_RAM_SIZE],

    interrupt_enable_register: u8,

    cartridge_data: Vec<u8>,
}

impl Mmu {
    pub fn new() -> Result<Self> {
        let work_ram: [u8; WORK_RAM_SIZE] = rand::random();
        Ok(Self {
            rom_bank_zero: [0; ROM_BANK_SIZE],
            rom_bank_swap: [0; ROM_BANK_SIZE],

            video_ram: [0; VIDEO_RAM_SIZE],
            external_ram: [0xFF; EXTERNAL_RAM_SIZE],
            work_ram,
            echo_ram: [0; ECHO_RAM_SIZE],

            sprite_attribution_table: [0; SPRITE_ATTRIBUTION_TABLE_SIZE],
            unusable: [0; UNUSABLE_MEMORY_SIZE],
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
        self.load_rom_bank(1);

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

    pub fn read_byte(&self, address: usize) -> Result<u8> {
        if address == 0xFF44 {
            return Ok(0x90);
        }

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
            Unusable => {
                let relative_address = address - Unusable as usize;
                self.unusable[relative_address]
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
            Unusable => {
                let relative_address = address - Unusable as usize;
                self.unusable[relative_address] = value;

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

    pub fn get_byte_ref(&mut self, address: usize) -> Result<&mut u8> {
        Ok(match MemoryRegion::from_address(address)? {
            RomBankZero => {
                let relative_address = address - RomBankZero as usize;
                &mut self.rom_bank_zero[relative_address]
            }
            RomBankSwap => {
                let relative_address = address - RomBankSwap as usize;
                &mut self.rom_bank_swap[relative_address]
            }
            VideoRam => {
                let relative_address = address - VideoRam as usize;
                &mut self.video_ram[relative_address]
            }
            ExternalRam => {
                let relative_address = address - ExternalRam as usize;
                &mut self.external_ram[relative_address]
            }
            WorkRam => {
                let relative_address = address - WorkRam as usize;
                &mut self.work_ram[relative_address]
            }
            EchoRam => {
                let relative_address = address - EchoRam as usize;
                &mut self.echo_ram[relative_address]
            }
            SpriteAttributionTable => {
                let relative_address = address - SpriteAttributionTable as usize;
                &mut self.sprite_attribution_table[relative_address]
            }
            Unusable => {
                let relative_address = address - Unusable as usize;
                &mut self.unusable[relative_address]
            }
            IO => {
                let relative_address = address - IO as usize;
                &mut self.io[relative_address]
            }
            HighRam => {
                let relative_address = address - HighRam as usize;
                &mut self.high_ram[relative_address]
            }
            InterruptEnableRegister => {
                &mut self.interrupt_enable_register
            }
        })
    }

    pub fn dump_memory_region(&self, region: MemoryRegion) -> Vec<u8> {
        match region {
            RomBankZero => self.rom_bank_zero.to_vec(),
            RomBankSwap => self.rom_bank_swap.to_vec(),
            VideoRam => self.video_ram.to_vec(),
            ExternalRam => self.external_ram.to_vec(),
            WorkRam => self.work_ram.to_vec(),
            EchoRam => self.echo_ram.to_vec(),
            SpriteAttributionTable => self.sprite_attribution_table.to_vec(),
            Unusable => self.unusable.to_vec(),
            IO => self.io.to_vec(),
            HighRam => self.high_ram.to_vec(),
            InterruptEnableRegister => vec![self.interrupt_enable_register]
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        [
            &self.rom_bank_zero[..],
            &self.rom_bank_swap[..],
            &self.video_ram[..],
            &self.external_ram[..],
            &self.work_ram[..],
            &self.echo_ram[..],
            &self.sprite_attribution_table[..],
            &self.io[..],
            &self.high_ram[..],
            std::slice::from_ref(&self.interrupt_enable_register),
        ].concat()
    }
}