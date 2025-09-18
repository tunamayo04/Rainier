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

    pub fn update_clock_cycles(&mut self, count: usize) {
        self.cycles += count;

        let tac = self.mmu.borrow().tac();
        if tac != 0xF8 && tac != 0 {
            println!("{}", tac);
        }
        let clock_enable = tac & (1 << 2) != 0;

        if clock_enable {
            let clock_select = tac & 0b11;

            match clock_select {
                0 => {
                    if self.cycles >= 256 {
                        self.cycles -= 256;
                        let tima = self.mmu.borrow().tima();

                        if tima == 0xFF {
                            let tma = self.mmu.borrow().tma();
                            self.mmu.borrow_mut().set_tima(tma);

                            let iflag = self.mmu.borrow().iflag();
                            self.mmu.borrow_mut().set_iflag(iflag | Interrupt::Timer as u8);
                        } else {
                            self.mmu.borrow_mut().set_tima(tima + 1);
                        }
                    }
                },
                1 => {

                },
                2 => {

                },
                3 => {

                }
                _ => {}
            }
        }
    }
}