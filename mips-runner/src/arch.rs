use crate::data::Data;
use crate::memory::{Memory, MemoryManager, PointerSizeT};
use crate::registers::{RegisterInfo, Registers, StackRegister};
use crate::utils::align;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use goblin::container::Endian;
use std::thread::available_parallelism;
use unicorn_engine::unicorn_const::{uc_error, Mode};
use unicorn_engine::{RegisterMIPS, Unicorn};

pub trait ArchT {
    fn endian(&self) -> Endian;
    fn pointer_size(&self) -> PointerSizeT;
    fn pc_reg_id(&self) -> i32;
    fn sp_reg_id(&self) -> i32;
    fn arch(&self) -> unicorn_engine::unicorn_const::Arch;
    fn mode(&self) -> Mode;
}

#[derive(Copy, Eq, PartialEq, Debug, Clone)]
pub struct ArchInfo {
    pub endian: Endian,
    pub pointer_size: u8,
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
    fn pointer_size(&self) -> u8 {
        if self.mode32 {
            4
        } else {
            8
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
                pointer_size: arch.pointer_size(),
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

    fn pc_reg_id(&self) -> i32 {
        self.get_data().register_info.pc
    }

    fn sp_reg_id(&self) -> i32 {
        self.get_data().register_info.sp
    }

    fn pointer_size(&self) -> u8 {
        self.get_data().pointersize()
    }
}

/// Stack operations
pub trait Stack: StackRegister + Memory + ArchT {
    /// Push a value onto the stack.
    /// return the top stack address after pushing the value
    fn stack_push(&mut self, pointer: u64) -> Result<u64, uc_error> {
        let ps = self.pointer_size();
        let new_sp = self.incr_sp(-(ps as i64))?;
        self.write_ptr(new_sp, pointer, Some(ps))?;
        Ok(new_sp)
    }

    /// Pop a value from stack.
    /// Returns: the value at the top.
    fn stack_pop(&mut self) -> Result<u64, uc_error> {
        let ps = self.pointer_size();
        let v = self.read_ptr(self.sp()?, Some(ps))?;
        self.incr_sp(ps as i64)?;
        Ok(v)
    }

    /// Peek the architectural stack at a specified offset from its top, without affecting the top of the stack.
    /// Note that this operation violates the FIFO property of the stack and may be used cautiously.
    ///        Args:
    ///             offset: offset in bytes from the top of the stack, not necessarily aligned to the
    ///                     native stack item size. the offset may be either positive or netagive, where
    ///                     a 0 value means retrieving the value at the top of the stack
    ///
    ///         Returns: the value at the specified address
    fn stack_read(&self, offset: i64) -> Result<u64, uc_error> {
        let addr = self
            .sp()?
            .checked_add_signed(offset)
            .ok_or_else(|| uc_error::EXCEPTION)?;
        let v = self.read_ptr(addr, None)?;
        Ok(v)
    }
    fn stack_write(&mut self, offset: i64, value: u64) -> Result<(), uc_error> {
        let addr = self
            .sp()?
            .checked_add_signed(offset)
            .ok_or_else(|| uc_error::EXCEPTION)?;
        self.write_ptr(addr, value, None)?;
        Ok(())
    }

    fn aligned_push_str(&mut self, s: &str) -> Result<u64, uc_error> {
        let mut b = s.as_bytes().to_vec();
        // add a 0x00 separator.
        b.push(0);
        self.aligned_push_bytes(&b, None)
    }

    /// alignment default to pointer-size.
    fn aligned_push_bytes(
        &mut self,
        s: impl AsRef<[u8]>,
        alignment: Option<u32>,
    ) -> Result<u64, uc_error> {
        let alignment = alignment.unwrap_or_else(|| self.pointer_size() as u32);
        let data = s.as_ref();
        let top = self.sp()?;
        // align by pointer size
        let top = align((top - data.len() as u64) as u32, alignment) as u64;
        Memory::write(self, top, data)?;
        self.set_sp(top)?;
        Ok(top)
    }
}

impl<'a> Stack for Unicorn<'a, Data> {}

pub struct Packer {
    endian: Endian,
    pointer_size: usize,
}

impl Packer {
    pub fn new(endian: Endian, pointer_size: PointerSizeT) -> Self {
        Self {
            endian,
            pointer_size: pointer_size as usize,
        }
    }
    pub fn pack(&self, v: u64) -> Vec<u8> {
        let mut buf = BytesMut::new();
        match self.endian {
            Endian::Little => {
                buf.put_uint_le(v, self.pointer_size);
            }

            Endian::Big => {
                buf.put_uint(v, self.pointer_size);
            }
        }
        buf.to_vec()
    }
    pub fn unpack(&self, data: Vec<u8>) -> u64 {
        let mut data = Bytes::from(data);

        match self.endian {
            Endian::Little => data.get_uint_le(self.pointer_size),
            Endian::Big => data.get_uint(self.pointer_size),
        }
    }
}
