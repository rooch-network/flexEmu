use crate::engine::Engine;
use log::Level::Debug;
use std::collections::BTreeMap;

use crate::arch::ArchT;
use unicorn_engine::unicorn_const::uc_error;

pub trait Registers {
    fn read(&self, reg: impl Into<i32>) -> Result<u64, uc_error>;
    fn write(&mut self, reg: impl Into<i32>, value: u64) -> Result<(), uc_error>;
    fn pc(&self) -> Result<u64, uc_error>;
    fn set_pc(&mut self, value: u64) -> Result<(), uc_error>;
    fn save_registers(&self) -> Result<BTreeMap<i32, u64>, uc_error>;
}
pub trait StackRegister {
    fn sp(&self) -> Result<u64, uc_error>;
    fn set_sp(&mut self, value: u64) -> Result<(), uc_error>;

    /// increment stack pointer by `delta`.
    /// Return new stack pointer
    fn incr_sp(&mut self, delta: i64) -> Result<u64, uc_error> {
        let cur = self.sp()?;
        let new_sp = cur.checked_add_signed(delta).ok_or(uc_error::EXCEPTION)?;
        self.set_sp(new_sp)?;
        Ok(new_sp)
    }
}

impl<'a, A: ArchT> StackRegister for Engine<'a, A> {
    fn sp(&self) -> Result<u64, uc_error> {
        self.read(A::SP)
    }

    fn set_sp(&mut self, value: u64) -> Result<(), uc_error> {
        self.write(A::SP, value)
    }
}

impl<'a, A: ArchT> Registers for Engine<'a, A> {
    fn read(&self, reg: impl Into<i32>) -> Result<u64, uc_error> {
        self.reg_read(reg)
    }
    fn write(&mut self, reg: impl Into<i32>, value: u64) -> Result<(), uc_error> {
        self.reg_write(reg, value)
    }
    fn pc(&self) -> Result<u64, uc_error> {
        self.read(A::PC)
    }

    fn set_pc(&mut self, value: u64) -> Result<(), uc_error> {
        self.write(A::PC, value)
    }
    fn save_registers(&self) -> Result<BTreeMap<i32, u64>, uc_error> {
        let mut reg_values = BTreeMap::default();
        for reg in self.get_data().env().registers() {
            let reg_v = Registers::read(self, *reg)?;
            if reg_v != 0 {
                reg_values.insert(*reg, reg_v);
            }
        }
        Ok(reg_values)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RegisterInfo {
    pub(crate) pc: i32,
    pub(crate) sp: i32,
}

impl RegisterInfo {
    pub fn new(pc_reg: impl Into<i32>, sp_reg: impl Into<i32>) -> Self {
        Self {
            pc: pc_reg.into(),
            sp: sp_reg.into(),
        }
    }
}

pub type RegisterState = BTreeMap<i32, u64>;
