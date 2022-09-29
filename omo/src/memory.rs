use anyhow::Result;

use crate::arch::ArchInfo;

use crate::{engine::Machine, utils::Packer, PAGE_SIZE};
use unicorn_engine::{
    unicorn_const::{uc_error, MemRegion, Permission},
    Unicorn,
};

pub type PointerSizeT = u8;
#[derive(Debug)]
struct MapInfo {
    info: MemRegion,
    label: String,
}

#[derive(Default, Debug)]
pub struct MemoryManager {
    map_info: Vec<MapInfo>,
}

impl MemoryManager {
    pub(crate) fn add_mapinfo(&mut self, mem_info: MemRegion, label: String) {
        self.map_info.push(MapInfo {
            info: mem_info,
            label,
        });
        self.map_info.sort_by_key(|info| info.info.begin);
    }
}

pub trait Memory {
    fn pagesize(&self) -> u64 {
        PAGE_SIZE as u64
    }
    fn mem_map(&mut self, region: MemRegion, info: Option<String>) -> Result<(), uc_error>;
    fn mem_unmap(&mut self, addr: u64, size: usize) -> Result<(), uc_error>;
    fn is_mapped(&self, addr: u64, size: usize) -> Result<bool, uc_error>;
    fn mprotect(&mut self, addr: u64, size: usize, perm: Permission) -> Result<(), uc_error>;
    fn read(&self, addr: u64, size: usize) -> Result<Vec<u8>, uc_error>;
    fn read_ptr(&self, address: u64, pointersize: Option<PointerSizeT>) -> Result<u64, uc_error>;

    fn write(&mut self, address: u64, bytes: impl AsRef<[u8]>) -> Result<(), uc_error>;
    /// Write an integer value to a memory address.
    /// Bytes written will be packed using emulated architecture properties.
    ///
    /// Args:
    ///  addr: target memory address
    ///  value: integer value to write
    ///  size: pointer size (in bytes): either 1, 2, 4, 8, or 0 for arch native size
    fn write_ptr(
        &mut self,
        address: u64,
        value: u64,
        pointersize: Option<PointerSizeT>,
    ) -> Result<(), uc_error>;
    //fn align_up(&self, value: u64, alignment: Option<usize>) -> u64;
}

impl<'a, A> Memory for Unicorn<'a, Machine<A>> {
    fn mem_map(
        &mut self,
        MemRegion { begin, end, perms }: MemRegion,
        info: Option<String>,
    ) -> Result<(), uc_error> {
        debug_assert!(
            perms & (!Permission::ALL) == Permission::NONE,
            "unexcepted permissions mask {:?}",
            perms
        );

        Unicorn::mem_map(self, begin, (end - begin) as usize, perms)?;
        self.get_data_mut().memories.add_mapinfo(
            MemRegion { begin, end, perms },
            info.unwrap_or_else(|| "[mapped]".to_string()),
        );
        Ok(())
    }
    fn mem_unmap(&mut self, addr: u64, size: usize) -> Result<(), uc_error> {
        // TODO: manage map_info
        Unicorn::mem_unmap(self, addr, size)
    }
    fn mprotect(&mut self, addr: u64, size: usize, perm: Permission) -> Result<(), uc_error> {
        // TODO: manage map_info
        Unicorn::mem_protect(self, addr, size, perm)
    }
    fn is_mapped(&self, _addr: u64, _size: usize) -> Result<bool, uc_error> {
        // FIXME: impl it.
        Ok(false)
    }

    fn read(&self, addr: u64, len: usize) -> Result<Vec<u8>, uc_error> {
        self.mem_read_as_vec(addr, len)
    }
    fn read_ptr(&self, address: u64, pointersize: Option<PointerSizeT>) -> Result<u64, uc_error> {
        let pointersize = pointersize.unwrap_or_else(|| self.pointer_size());
        let data = Memory::read(self, address, pointersize as usize)?;
        let packer = Packer::new(self.endian(), pointersize);
        Ok(packer.unpack(data))
    }
    fn write(&mut self, address: u64, bytes: impl AsRef<[u8]>) -> Result<(), uc_error> {
        self.mem_write(address, bytes.as_ref())?;
        self.get_data_mut()
            .state
            .memory
            .write_bytes(address, bytes.as_ref());
        Ok(())
    }

    fn write_ptr(
        &mut self,
        address: u64,
        value: u64,
        pointersize: Option<PointerSizeT>,
    ) -> Result<(), uc_error> {
        let pointersize = pointersize.unwrap_or_else(|| self.pointer_size());

        let packer = Packer::new(self.endian(), pointersize);
        Memory::write(self, address, packer.pack(value))
    }
}
