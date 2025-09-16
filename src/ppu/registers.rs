use crate::ppu::Ppu;

pub enum Register {
    BGEnable = 0,
    SpriteEnable = 1,
    SpriteSize = 2,
    BGTileMapSelect = 3,
    TileDataSelect = 4,
    WindowDisplayEnable = 5,
    WindowTileMapSelect = 6,
    DisplayEnable = 7,
}

impl Ppu {
    pub(crate) fn check_register(&self, register: Register) -> bool {
        let mmu = self.mmu.borrow();
        let register_value = mmu.lcdc() & (1 << register as u8);

        register_value != 0
    }

    pub(crate) fn set_register(&mut self, register: Register, value: bool) {
        let mut mmu = self.mmu.borrow_mut();

        if value {
            let lcdc = mmu.lcdc() | (1 << register as u8);
            mmu.set_lcdc(lcdc);
        }
        else {
            let lcdc = mmu.lcdc() & !(1 << register as u8);
            mmu.set_lcdc(lcdc);
        }
    }
}