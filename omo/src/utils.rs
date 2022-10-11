use anyhow::Error;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use goblin::{
    container::Endian,
    elf::program_header::{PF_R, PF_W, PF_X},
};
use num_traits::PrimInt;
use std::fmt::Debug;

use crate::{
    arch::ArchT,
    engine::Engine,
    errors::EmulatorError,
    memory::{Memory, PointerSizeT},
};
use unicorn_engine::unicorn_const::Permission;

/// Align a value down to the specified alignment boundary. If `value` is already
/// aligned, the same value is returned. Commonly used to determine the base address
/// of the enclosing page.
///
/// Args:
/// value: a value to align
/// alignment: alignment boundary; must be a power of 2. if not specified value
/// will be aligned to page size
///
/// Returns: value aligned down to boundary
pub fn align<T>(value: T, alignment: impl Into<T>) -> T
where
    T: PrimInt + Debug,
{
    let alignment = alignment.into();
    let mask = alignment - T::one();
    debug_assert_eq!(alignment & mask, T::zero());
    // round down to nearest alignment
    value & (!mask)
}

// pub fn align_up(value: u32, alignment: u32) -> u32 {
//     debug_assert_eq!(alignment & (alignment - 1), 0);
//     // round up to nearest alignment
//     (value + alignment - 1) & (!(alignment - 1))
// }

pub fn align_up<T>(value: T, alignment: impl Into<T>) -> T
where
    T: PrimInt + Debug,
{
    let alignment = alignment.into();
    let mask = alignment - T::one();
    debug_assert_eq!(alignment & mask, T::zero());
    // round up to nearest alignment
    (value + mask) & (!mask)
}

/// Translate ELF segment perms to Unicorn protection constants.
pub fn seg_perm_to_uc_prot(perm: u32) -> Permission {
    let mut prot = Permission::NONE;
    if perm & PF_X != 0 {
        prot |= Permission::EXEC;
    }
    if perm & PF_W != 0 {
        prot |= Permission::WRITE;
    }
    if perm & PF_R != 0 {
        prot |= Permission::READ;
    }

    prot
}

pub fn read_string<'a, A: ArchT>(
    core: &mut Engine<'a, A>,
    address: u64,
    terminator: &[u8],
) -> Result<String, EmulatorError> {
    let mut result: Vec<u8> = Vec::new();
    let char_len = terminator.len();

    let mut char = Memory::read(core, address, char_len)?;

    let mut address = address;
    let terminator = terminator.to_vec();
    while char != terminator {
        address += char_len as u64;
        result.extend(char.clone());
        char = Memory::read(core, address, char_len)?;
    }
    let result = match String::from_utf8(char) {
        Ok(r) => r,
        Err(e) => return Err(EmulatorError::Custom(Error::new(e))),
    };

    Ok(result)
}

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

#[cfg(test)]
mod test {
    use super::{align, align_up};

    #[test]
    pub fn test_align() {
        let pagesize = 0x1000;

        {
            assert_eq!(align(0x0111, pagesize), 0x0000);
            assert_eq!(align(0x1000, pagesize), 0x1000);
            assert_eq!(align(0x1001, pagesize), 0x1000);
            assert_eq!(align(0x1111, pagesize), 0x1000);
            assert_eq!(align(0x10000, pagesize), 0x10000);
        }

        {
            assert_eq!(align_up(0x0111, pagesize), 0x1000);
            assert_eq!(align_up(0x1000, pagesize), 0x1000);
            assert_eq!(align_up(0x1001, pagesize), 0x2000);
            assert_eq!(align_up(0x1111, pagesize), 0x2000);
            assert_eq!(align_up(0x2000, pagesize), 0x2000);
            assert_eq!(align_up(0x10000, pagesize), 0x10000);
        }
    }
}
