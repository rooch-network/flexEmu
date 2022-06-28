use crate::data::Data;
use unicorn_engine::unicorn_const::uc_error;
use unicorn_engine::Unicorn;

pub trait Registers {
    fn read(&self, reg: impl Into<i32>) -> Result<u64, uc_error>;
    fn write(&mut self, reg: impl Into<i32>, value: u64) -> Result<(), uc_error>;
    fn pc(&self) -> Result<u64, uc_error>;
    fn sp(&self) -> Result<u64, uc_error>;
    fn set_pc(&mut self, value: u64) -> Result<(), uc_error>;
    fn set_sp(&mut self, value: u64) -> Result<(), uc_error>;
    fn incr_sp(&mut self, delta: i64) -> Result<(), uc_error> {
        let cur = self.sp()?;
        let new_sp = cur
            .checked_add_signed(delta)
            .ok_or_else(|| uc_error::EXCEPTION)?;
        self.set_sp(new_sp)
    }
}

impl<'a> Registers for Unicorn<'a, Data> {
    fn read(&self, reg: impl Into<i32>) -> Result<u64, uc_error> {
        self.reg_read(reg)
    }
    fn write(&mut self, reg: impl Into<i32>, value: u64) -> Result<(), uc_error> {
        self.reg_write(reg, value)
    }
    fn pc(&self) -> Result<u64, uc_error> {
        let pc_reg = self.get_data().register_info.pc;
        self.read(pc_reg)
    }

    fn sp(&self) -> Result<u64, uc_error> {
        let sp_reg = self.get_data().register_info.sp;
        self.read(sp_reg)
    }

    fn set_pc(&mut self, value: u64) -> Result<(), uc_error> {
        let pc_reg = self.get_data().register_info.pc;
        self.write(pc_reg, value)
    }
    fn set_sp(&mut self, value: u64) -> Result<(), uc_error> {
        let sp_reg = self.get_data().register_info.sp;
        self.write(sp_reg, value)
    }
}

pub struct RegisterInfo {
    pc: i32,
    sp: i32,
}

impl RegisterInfo {
    pub fn new(pc_reg: impl Into<i32>, sp_reg: impl Into<i32>) -> Self {
        Self {
            pc: pc_reg.into(),
            sp: sp_reg.into(),
        }
    }
}
