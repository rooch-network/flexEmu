use std::cell::RefCell;
use std::rc::Rc;
use unicorn_engine::{Unicorn};
use unicorn_engine::unicorn_const::uc_error;

pub struct RegisterManager<'a> {
    uc: Rc<RefCell<Unicorn<'a, ()>>>,
    pc: i32,
    sp: i32
}


impl<'a> RegisterManager<'a> {
    pub fn new(uc: Rc<RefCell<Unicorn<'a, ()>>>, pc_reg: impl Into<i32>, sp_reg: impl Into<i32>) -> Self {
        Self {
            uc,
            pc: pc_reg.into(),
            sp: sp_reg.into()
        }
    }

    pub fn read(&self, reg: impl Into<i32>) -> Result<u64, uc_error> {
        self.uc.borrow().reg_read(reg)
    }
    pub fn write(&mut self, reg: impl Into<i32>, value: u64) -> Result<(), uc_error> {
        self.uc.borrow_mut().reg_write(reg, value)
    }
    pub fn pc(&self) -> Result<u64, uc_error> {
        self.read(self.pc)
    }

    pub fn sp(&self) -> Result<u64, uc_error> {
        self.read(self.sp)
    }

    pub fn set_pc(&mut self, value: u64) -> Result<(), uc_error> {
        self.write(self.pc, value)
    }
    pub fn set_sp(&mut self, value: u64) -> Result<(), uc_error> {
        self.write(self.sp, value)
    }
}
