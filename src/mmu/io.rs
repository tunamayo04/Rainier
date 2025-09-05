use crate::mmu::Mmu;

impl Mmu {
    // --- Joypad $FF00 (Mixed) ---
    pub fn p1(&self) -> u8 { self.read_byte(0xFF00).unwrap() }
    pub fn set_p1(&mut self, val: u8) { self.write_byte(0xFF00, val).unwrap() }

    // --- Serial $FF01-$FF02 ---
    pub fn sb(&self) -> u8 { self.read_byte(0xFF01).unwrap() }
    pub fn set_sb(&mut self, val: u8) { self.write_byte(0xFF01, val).unwrap() }

    pub fn sc(&self) -> u8 { self.read_byte(0xFF02).unwrap() }
    pub fn set_sc(&mut self, val: u8) { self.write_byte(0xFF02, val).unwrap() }

    // --- Timer ---
    pub fn div(&self) -> u8 { self.read_byte(0xFF04).unwrap() }
    pub fn set_div(&mut self, val: u8) { self.write_byte(0xFF04, val).unwrap() }

    pub fn tima(&self) -> u8 { self.read_byte(0xFF05).unwrap() }
    pub fn set_tima(&mut self, val: u8) { self.write_byte(0xFF05, val).unwrap() }

    pub fn tma(&self) -> u8 { self.read_byte(0xFF06).unwrap() }
    pub fn set_tma(&mut self, val: u8) { self.write_byte(0xFF06, val).unwrap() }

    pub fn tac(&self) -> u8 { self.read_byte(0xFF07).unwrap() }
    pub fn set_tac(&mut self, val: u8) { self.write_byte(0xFF07, val).unwrap() }

    // --- Interrupts ---
    pub fn iflag(&self) -> u8 { self.read_byte(0xFF0F).unwrap() }
    pub fn set_iflag(&mut self, val: u8) { self.write_byte(0xFF0F, val).unwrap() }

    pub fn ie(&self) -> u8 { self.read_byte(0xFFFF).unwrap() }
    pub fn set_ie(&mut self, val: u8) { self.write_byte(0xFFFF, val).unwrap() }

    // --- Sound registers ---
    pub fn nr10(&self) -> u8 { self.read_byte(0xFF10).unwrap() }
    pub fn set_nr10(&mut self, val: u8) { self.write_byte(0xFF10, val).unwrap() }

    pub fn nr11(&self) -> u8 { self.read_byte(0xFF11).unwrap() }
    pub fn set_nr11(&mut self, val: u8) { self.write_byte(0xFF11, val).unwrap() }

    pub fn nr12(&self) -> u8 { self.read_byte(0xFF12).unwrap() }
    pub fn set_nr12(&mut self, val: u8) { self.write_byte(0xFF12, val).unwrap() }

    pub fn set_nr13(&mut self, val: u8) { self.write_byte(0xFF13, val).unwrap() }

    pub fn nr14(&self) -> u8 { self.read_byte(0xFF14).unwrap() }
    pub fn set_nr14(&mut self, val: u8) { self.write_byte(0xFF14, val).unwrap() }

    pub fn nr21(&self) -> u8 { self.read_byte(0xFF16).unwrap() }
    pub fn set_nr21(&mut self, val: u8) { self.write_byte(0xFF16, val).unwrap() }

    pub fn nr22(&self) -> u8 { self.read_byte(0xFF17).unwrap() }
    pub fn set_nr22(&mut self, val: u8) { self.write_byte(0xFF17, val).unwrap() }

    pub fn set_nr23(&mut self, val: u8) { self.write_byte(0xFF18, val).unwrap() }

    pub fn nr24(&self) -> u8 { self.read_byte(0xFF19).unwrap() }
    pub fn set_nr24(&mut self, val: u8) { self.write_byte(0xFF19, val).unwrap() }

    pub fn nr30(&self) -> u8 { self.read_byte(0xFF1A).unwrap() }
    pub fn set_nr30(&mut self, val: u8) { self.write_byte(0xFF1A, val).unwrap() }

    pub fn set_nr31(&mut self, val: u8) { self.write_byte(0xFF1B, val).unwrap() }

    pub fn nr32(&self) -> u8 { self.read_byte(0xFF1C).unwrap() }
    pub fn set_nr32(&mut self, val: u8) { self.write_byte(0xFF1C, val).unwrap() }

    pub fn set_nr33(&mut self, val: u8) { self.write_byte(0xFF1D, val).unwrap() }

    pub fn nr34(&self) -> u8 { self.read_byte(0xFF1E).unwrap() }
    pub fn set_nr34(&mut self, val: u8) { self.write_byte(0xFF1E, val).unwrap() }

    pub fn set_nr41(&mut self, val: u8) { self.write_byte(0xFF20, val).unwrap() }

    pub fn nr42(&self) -> u8 { self.read_byte(0xFF21).unwrap() }
    pub fn set_nr42(&mut self, val: u8) { self.write_byte(0xFF21, val).unwrap() }

    pub fn nr43(&self) -> u8 { self.read_byte(0xFF22).unwrap() }
    pub fn set_nr43(&mut self, val: u8) { self.write_byte(0xFF22, val).unwrap() }

    pub fn nr44(&self) -> u8 { self.read_byte(0xFF23).unwrap() }
    pub fn set_nr44(&mut self, val: u8) { self.write_byte(0xFF23, val).unwrap() }

    pub fn nr50(&self) -> u8 { self.read_byte(0xFF24).unwrap() }
    pub fn set_nr50(&mut self, val: u8) { self.write_byte(0xFF24, val).unwrap() }

    pub fn nr51(&self) -> u8 { self.read_byte(0xFF25).unwrap() }
    pub fn set_nr51(&mut self, val: u8) { self.write_byte(0xFF25, val).unwrap() }

    pub fn nr52(&self) -> u8 { self.read_byte(0xFF26).unwrap() }
    pub fn set_nr52(&mut self, val: u8) { self.write_byte(0xFF26, val).unwrap() }

    // --- Wave RAM $FF30-FF3F ---
    pub fn wave_ram(&self, index: u8) -> u8 {
        assert!(index < 16, "Wave RAM index out of bounds");
        self.read_byte(0xFF30 + index as usize).unwrap()
    }
    pub fn set_wave_ram(&mut self, index: u8, val: u8) {
        assert!(index < 16, "Wave RAM index out of bounds");
        self.write_byte(0xFF30 + index as usize, val).unwrap()
    }

    // --- LCD / GPU ---
    pub fn lcdc(&self) -> u8 { self.read_byte(0xFF40).unwrap() }
    pub fn set_lcdc(&mut self, val: u8) { self.write_byte(0xFF40, val).unwrap() }

    pub fn stat(&self) -> u8 { self.read_byte(0xFF41).unwrap() }
    pub fn set_stat(&mut self, val: u8) { self.write_byte(0xFF41, val).unwrap() }

    pub fn scy(&self) -> u8 { self.read_byte(0xFF42).unwrap() }
    pub fn set_scy(&mut self, val: u8) { self.write_byte(0xFF42, val).unwrap() }

    pub fn scx(&self) -> u8 { self.read_byte(0xFF43).unwrap() }
    pub fn set_scx(&mut self, val: u8) { self.write_byte(0xFF43, val).unwrap() }


    pub fn ly(&self) -> u8 { self.read_byte(0xFF44).unwrap() }
    pub fn set_ly(&mut self, val: u8) { self.write_byte(0xFF44, val).unwrap() }

    pub fn lyc(&self) -> u8 { self.read_byte(0xFF45).unwrap() }
    pub fn set_lyc(&mut self, val: u8) { self.write_byte(0xFF45, val).unwrap() }

    pub fn dma(&self) -> u8 { self.read_byte(0xFF46).unwrap() }
    pub fn set_dma(&mut self, val: u8) { self.write_byte(0xFF46, val).unwrap() }

    pub fn bgp(&self) -> u8 { self.read_byte(0xFF47).unwrap() }
    pub fn set_bgp(&mut self, val: u8) { self.write_byte(0xFF47, val).unwrap() }

    pub fn obp0(&self) -> u8 { self.read_byte(0xFF48).unwrap() }
    pub fn set_obp0(&mut self, val: u8) { self.write_byte(0xFF48, val).unwrap() }

    pub fn obp1(&self) -> u8 { self.read_byte(0xFF49).unwrap() }
    pub fn set_obp1(&mut self, val: u8) { self.write_byte(0xFF49, val).unwrap() }

    pub fn wy(&self) -> u8 { self.read_byte(0xFF4A).unwrap() }
    pub fn set_wy(&mut self, val: u8) { self.write_byte(0xFF4A, val).unwrap() }

    pub fn wx(&self) -> u8 { self.read_byte(0xFF4B).unwrap() }
    pub fn set_wx(&mut self, val: u8) { self.write_byte(0xFF4B, val).unwrap() }

    // --- CGB Registers ---

    // CPU speed / mode
    pub fn key0(&self) -> u8 { self.read_byte(0xFF4C).unwrap() }
    pub fn set_key0(&mut self, val: u8) { self.write_byte(0xFF4C, val).unwrap() }

    pub fn key1(&self) -> u8 { self.read_byte(0xFF4D).unwrap() }
    pub fn set_key1(&mut self, val: u8) { self.write_byte(0xFF4D, val).unwrap() }

    // VRAM bank
    pub fn vbk(&self) -> u8 { self.read_byte(0xFF4F).unwrap() }
    pub fn set_vbk(&mut self, val: u8) { self.write_byte(0xFF4F, val).unwrap() }

    // Boot ROM mapping control (write-only)
    pub fn set_bank(&mut self, val: u8) { self.write_byte(0xFF50, val).unwrap() }

    // HDMA channels
    pub fn set_hdma1(&mut self, val: u8) { self.write_byte(0xFF51, val).unwrap() }

    pub fn set_hdma2(&mut self, val: u8) { self.write_byte(0xFF52, val).unwrap() }

    pub fn set_hdma3(&mut self, val: u8) { self.write_byte(0xFF53, val).unwrap() }

    pub fn set_hdma4(&mut self, val: u8) { self.write_byte(0xFF54, val).unwrap() }

    pub fn hdma5(&self) -> u8 { self.read_byte(0xFF55).unwrap() }
    pub fn set_hdma5(&mut self, val: u8) { self.write_byte(0xFF55, val).unwrap() }

    // Infrared communications port (Mixed)
    pub fn rp(&self) -> u8 { self.read_byte(0xFF56).unwrap() }
    pub fn set_rp(&mut self, val: u8) { self.write_byte(0xFF56, val).unwrap() }

    // Background palette (CGB)
    pub fn bcps(&self) -> u8 { self.read_byte(0xFF68).unwrap() }
    pub fn set_bcps(&mut self, val: u8) { self.write_byte(0xFF68, val).unwrap() }

    pub fn bcpd(&self) -> u8 { self.read_byte(0xFF69).unwrap() }
    pub fn set_bcpd(&mut self, val: u8) { self.write_byte(0xFF69, val).unwrap() }

    // OBJ palette (CGB)
    pub fn ocps(&self) -> u8 { self.read_byte(0xFF6A).unwrap() }
    pub fn set_ocps(&mut self, val: u8) { self.write_byte(0xFF6A, val).unwrap() }

    pub fn ocpd(&self) -> u8 { self.read_byte(0xFF6B).unwrap() }
    pub fn set_ocpd(&mut self, val: u8) { self.write_byte(0xFF6B, val).unwrap() }

    // Object priority mode
    pub fn opri(&self) -> u8 { self.read_byte(0xFF6C).unwrap() }
    pub fn set_opri(&mut self, val: u8) { self.write_byte(0xFF6C, val).unwrap() }

    // WRAM bank (CGB)
    pub fn svbk(&self) -> u8 { self.read_byte(0xFF70).unwrap() }
    pub fn set_svbk(&mut self, val: u8) { self.write_byte(0xFF70, val).unwrap() }

    // PCM output (read-only)
    pub fn pcm12(&self) -> u8 { self.read_byte(0xFF76).unwrap() }

    pub fn pcm34(&self) -> u8 { self.read_byte(0xFF77).unwrap() }
    
    pub fn ime(&self) -> u8 { self.read_byte(0xFFFF).unwrap() }
    pub fn set_ime(&mut self, val: u8) { self.write_byte(0xFF78, val).unwrap() }
}