use crate::data::Data;
use crate::memory::{Memory, MemoryManager};
use crate::registers::{RegisterInfo, Registers, StackRegister};
use crate::utils::align;
use bytes::{BufMut, BytesMut};
use goblin::container::Endian;
use std::thread::available_parallelism;
use unicorn_engine::unicorn_const::{uc_error, Mode};
use unicorn_engine::{RegisterMIPS, Unicorn};

pub trait ArchT {
    fn endian(&self) -> Endian;
    fn bit(&self) -> u64;
    fn pc_reg_id(&self) -> i32;
    fn sp_reg_id(&self) -> i32;
    fn arch(&self) -> unicorn_engine::unicorn_const::Arch;
    fn mode(&self) -> Mode;
}

#[derive(Copy, Eq, PartialEq, Debug, Clone)]
pub struct ArchInfo {
    pub endian: Endian,
    pub bit: u64,
    pub mode: Mode,
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
}

impl<'a> Core<'a> {
    pub fn new(arch: impl ArchT) -> Self {
        let data = Data {
            register_info: RegisterInfo::new(arch.pc_reg_id(), arch.sp_reg_id()),
            memories: MemoryManager::default(),
            arch_info: ArchInfo {
                endian: arch.endian(),
                bit: arch.bit(),
                mode: arch.mode(),
            },
        };
        let uc = Unicorn::new_with_data(arch.arch(), arch.mode(), data).unwrap();
        Self { uc }
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
        self.uc.get_data().pointersize()
    }
    pub fn endian(&self) -> Endian {
        self.uc.get_data().endian()
    }
    pub fn arch(&self) -> unicorn_engine::unicorn_const::Arch {
        self.uc.get_arch()
    }

    pub fn stack_push(&mut self, value: u64) -> u64 {
        self.uc.stack_push(value)
    }
    pub fn stack_pop(&mut self) -> u64 {
        self.uc.stack_pop()
    }
}

impl<'a> ArchT for Unicorn<'a, Data> {
    fn endian(&self) -> Endian {
        self.get_data().endian()
    }
    fn arch(&self) -> unicorn_engine::unicorn_const::Arch {
        self.get_arch()
    }
    fn mode(&self) -> Mode {
        self.get_data().arch_info.mode
    }
    fn bit(&self) -> u64 {
        self.get_data().arch_info.bit
    }

    fn pc_reg_id(&self) -> i32 {
        self.get_data().register_info.pc
    }

    fn sp_reg_id(&self) -> i32 {
        self.get_data().register_info.sp
    }
}

/// Stack operations
pub trait Stack: StackRegister {
    /// Push a value onto the stack.
    /// return the top stack address after pushing the value
    fn stack_push(&mut self, value: u64) -> u64 {
        unimplemented!()
    }
    fn stack_pop(&mut self) -> u64 {
        unimplemented!()
    }
    fn stack_read(&self, offset: u64) -> u64 {
        unimplemented!()
    }
    fn stack_write(&mut self, offset: u64, value: u64) {
        unimplemented!()
    }
    fn push_str(&mut self, s: &str) -> Result<u64, uc_error>
    where
        Self: ArchT + Memory,
    {
        let mut b = s.as_bytes().to_vec();
        // add a 0x00 separator.
        b.push(0);
        self.push_bytes(&b)
    }
    fn push_bytes(&mut self, s: impl AsRef<[u8]>) -> Result<u64, uc_error>
    where
        Self: ArchT + Memory,
    {
        let data = s.as_ref();
        let top = self.sp()?;
        // align by pointer size
        let top = align((top - data.len() as u64) as u32, (self.bit() / 8) as u32) as u64;
        Memory::write(self, top, data)?;
        self.set_sp(top)?;
        Ok(top)
    }
}

/// TODO: impl me
impl<'a> Stack for Unicorn<'a, Data> {}

pub struct Packer {
    endian: Endian,
    pointsize: usize,
}

impl Packer {
    pub fn new(endian: Endian, pointsize: usize) -> Self {
        Self { endian, pointsize }
    }
    pub fn pack(&self, v: u64) -> Vec<u8> {
        let mut buf = BytesMut::new();
        match self.endian {
            Endian::Little => {
                buf.put_uint_le(v, self.pointsize);
            }

            Endian::Big => {
                buf.put_uint(v, self.pointsize);
            }
        }
        buf.to_vec()
    }
}
