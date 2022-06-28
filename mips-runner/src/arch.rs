use crate::data::Data;
use crate::memory::MemoryManager;
use crate::registers::RegisterInfo;
use goblin::container::Endian;
use unicorn_engine::unicorn_const::Mode;
use unicorn_engine::{RegisterMIPS, Unicorn};

pub trait ArchT {
    fn endian(&self) -> Endian;
    fn bit(&self) -> u64;
    fn pc_reg_id(&self) -> i32;
    fn sp_reg_id(&self) -> i32;
    fn arch(&self) -> unicorn_engine::unicorn_const::Arch;
    fn mode(&self) -> Mode;
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
    fn arch(&self) -> unicorn_engine::unicorn_const::Arch {
        unicorn_engine::unicorn_const::Arch::MIPS
    }
    fn mode(&self) -> Mode {
        let mut mode = if self.mode32 {
            Mode::MODE_32
        } else {
            Mode::MODE_64
        };
        match self.endian {
            Endian::Little => {
                mode |= Mode::LITTLE_ENDIAN;
            }
            Endian::Big => {
                mode |= Mode::BIG_ENDIAN;
            }
        };
        mode
    }
    fn bit(&self) -> u64 {
        if self.mode32 {
            32
        } else {
            64
        }
    }

    fn pc_reg_id(&self) -> i32 {
        RegisterMIPS::PC as i32
    }

    fn sp_reg_id(&self) -> i32 {
        RegisterMIPS::SP as i32
    }
}

pub struct Core<'a> {
    uc: Unicorn<'a, Data>,
    endian: Endian,
    bits: u64,
}

impl<'a> Core<'a> {
    pub fn new(arch: impl ArchT) -> Self {
        let data = Data {
            register_info: RegisterInfo::new(arch.pc_reg_id(), arch.sp_reg_id()),
            memories: MemoryManager::default(),
        };
        let uc = Unicorn::new_with_data(arch.arch(), arch.mode(), data).unwrap();
        Self {
            uc,
            endian: arch.endian(),
            bits: arch.bit(),
        }
    }
    pub fn uc_mut(&mut self) -> &mut Unicorn<'a, Data> {
        &mut self.uc
    }

    pub fn uc(&self) -> &Unicorn<'a, Data> {
        &self.uc
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
    pub fn arch(&self) -> unicorn_engine::unicorn_const::Arch {
        self.uc.get_arch()
    }
    pub fn stack_push(&mut self, value: u64) -> u64 {
        unimplemented!()
    }
    pub fn stack_pop(&mut self) -> u64 {
        unimplemented!()
    }
}
