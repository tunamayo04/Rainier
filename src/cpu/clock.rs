use std::cell::RefCell;
use std::rc::Rc;
use crate::cpu::interrupts::Interrupt;
use crate::mmu::Mmu;

pub struct Clock {
    mmu: Rc<RefCell<Mmu>>,
    cycles: usize
}

impl Clock {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        Self {
            mmu,
            cycles: 0
        }
    }

    pub fn update_clock_cycles(&mut self, count: u8) {
        self.cycles += count as usize;

        let tac = self.mmu.borrow().tac();
        let clock_enable = tac & (1 << 2) != 0;

        if clock_enable {
            let clock_select = tac & 0b11;

            let cycle_increment = match clock_select {
                0 => 256,
                1 => 4,
                2 => 16,
                3 => 64,
                _ => 256
            };

            if self.cycles >= cycle_increment {
                self.cycles -= cycle_increment;
                let tima = self.mmu.borrow().tima();

                if tima == 0xFF {
                    let tma = self.mmu.borrow().tma();
                    self.mmu.borrow_mut().set_tima(tma);

                    let iflag = self.mmu.borrow().iflag();
                    self.mmu.borrow_mut().set_iflag(iflag | (1 << Interrupt::Timer as u8));
                } else {
                    self.mmu.borrow_mut().set_tima(tima + 1);
                }
            }
        };
    }
}