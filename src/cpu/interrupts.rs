use std::cell::RefCell;
use std::rc::Rc;
use crate::mmu::Mmu;

pub struct Interrupts {
    mmu: Rc<RefCell<Mmu>>,
}

impl Interrupts {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        Interrupts { mmu }
    }

    pub fn get_interrupt_master_enable(&self) -> bool {
        self.mmu.borrow().ime() == 1
    }
}