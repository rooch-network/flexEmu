use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use goblin::container::Endian;
use unicorn_engine::{RegisterMIPS, Unicorn};
use unicorn_engine::unicorn_const::Mode;
use crate::{MemoryManager, RegisterManager};

pub trait ArchT {
    fn endian(&self) -> Endian;
    fn bit(&self) -> u64;
    fn get_uc<'a>(&self) -> Unicorn<'a, ()>;
    fn pc_reg_id(&self) -> i32;
    fn sp_reg_id(&self) -> i32;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ArchMIPS {
    mode32: bool,
    endian: Endian,
}
impl Default for ArchMIPS {
    fn default() -> Self {
        Self {
            mode32: true,
            endian: Endian::Big,
        }
    }
}

impl ArchT for ArchMIPS {
    fn endian(&self) -> Endian {
        self.endian
    }

    fn bit(&self) -> u64 {
        if self.mode32 {
            32
        }else {
            64
        }

    }

    fn get_uc<'a>(&self) -> Unicorn<'a, ()> {
        let mut mode = if self.mode32 {
            Mode::MODE_32
        } else {
            Mode::MODE_64

        };
        match self.endian {
            Endian::Little => { mode |= Mode::LITTLE_ENDIAN;},
            Endian::Big => {
                mode |= Mode::BIG_ENDIAN;
            }
        }
        Unicorn::new(unicorn_engine::unicorn_const::Arch::MIPS, mode).unwrap()
    }

    fn pc_reg_id(&self) -> i32 {
        RegisterMIPS::PC as i32
    }

    fn sp_reg_id(&self) -> i32 {
        RegisterMIPS::SP as i32
    }
}

pub struct Arch<'a> {
    uc: Rc<RefCell<Unicorn<'a, ()>>>,
    pub(crate) mem: MemoryManager<'a>,
    pub(crate) registers: RegisterManager<'a>,
    endian: Endian,
    bits: u64,
}

impl<'a> Arch<'a> {
    pub fn new(arch: impl ArchT) -> Self {
        let uc = Rc::new(RefCell::new(arch.get_uc()));
        let mem = MemoryManager::new(uc.clone());
        let registers = RegisterManager::new(uc.clone(), arch.pc_reg_id(), arch.sp_reg_id());
        Self {
            uc,
            mem,
            registers,
            endian: arch.endian(),
            bits: arch.bit()
        }
    }
    pub fn uc_mut(&mut self,) -> &mut Unicorn<'a, ()> {
        self.uc.get_mut()
    }
    pub fn uc(&self) -> &Unicorn<'a, ()> {
        self.uc.borrow().deref()
    }
    // pub fn registers_mut(&mut self,) -> &mut Unicorn<'a, ()> {
    //     self.uc.get_mut()
    // }
    // pub fn registers(&self) -> &Unicorn<'a, ()> {
    //     self.registers.borrow().deref()
    // }
    pub fn pointersize(&self) -> u64 {
        self.bits / 8
    }
    pub fn endian(&self) -> Endian {
        self.endian
    }

    pub fn stack_push(&mut self, value: u64) -> u64 {
        unimplemented!()
    }
    pub fn stack_pop(&mut self) -> u64 {
        unimplemented!()
    }
}
