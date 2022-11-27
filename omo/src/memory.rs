use std::borrow::BorrowMut;

use anyhow::Result;
use log::info;
use unicorn_engine::{
    unicorn_const::{uc_error, MemRegion, Permission},
    Unicorn,
};

use crate::{
    arch::ArchInfo,
    engine::Machine,
    errors::{from_raw_syscall_ret, EmulatorError},
    utils::Packer,
    PAGE_SIZE,
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
    // Query whether the memory range starting at `addr` and is of length of `size` bytes
    // is fully mapped.
    //
    // Returns: True if the specified memory range is taken fully, False otherwise
    fn is_mapped(&self, addr: u64, size: usize) -> bool;
    // Choose mmap address by size.
    // Start searching with mmap base address in config.
    fn next_mmap_address(&self, base_address: u64, size: usize) -> Result<u64, EmulatorError>;
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
        log::debug!("mmapped: {:?}", self.get_data().memories.map_info);
        Ok(())
    }
    fn mem_unmap(&mut self, addr: u64, size: usize) -> Result<(), uc_error> {
        let begin = addr;
        let end = addr + size as u64;
        let mut wait_rm: Vec<MemRegion> = Vec::new();
        let mut wait_add: Vec<MapInfo> = Vec::new();

        let mut unmap_begin = begin;

        for mut i in &self.get_data_mut().memories.map_info {
            if unmap_begin < i.info.begin {
                unmap_begin = i.info.begin // illegal range -> illegal range.
            }
            if unmap_begin >= end {
                break; // all marked.
            }
            if unmap_begin >= i.info.end {
                continue; // no overlap in this range, try next.
            }
            if unmap_begin == i.info.begin {
                wait_rm.push(MemRegion {
                    begin: i.info.begin,
                    end: i.info.end,
                    perms: i.info.perms,
                });
                if end < i.info.end {
                    wait_add.push(MapInfo {
                        info: MemRegion {
                            begin: end,
                            end: i.info.end,
                            perms: i.info.perms,
                        },
                        label: i.label.clone(),
                    });
                };
            } else {
                // unmap_begin > i.info.begin && unmap_begin < i.info.end
                wait_rm.push(MemRegion {
                    begin: i.info.begin,
                    end: i.info.end,
                    perms: i.info.perms,
                });
                wait_add.push(MapInfo {
                    info: MemRegion {
                        begin: i.info.begin,
                        end: unmap_begin,
                        perms: i.info.perms,
                    },
                    label: i.label.clone(),
                });
                if end < i.info.end {
                    wait_add.push(MapInfo {
                        info: MemRegion {
                            begin: end,
                            end: i.info.end,
                            perms: i.info.perms,
                        },
                        label: i.label.clone(),
                    });
                }
            }
        }

        for ri in wait_rm {
            self.get_data_mut()
                .memories
                .map_info
                .retain(|i| !(i.info.begin == ri.begin && i.info.end == ri.end))
        }
        for ai in wait_add {
            self.get_data_mut().memories.map_info.push(ai)
        }
        self.get_data_mut()
            .memories
            .map_info
            .sort_by_key(|info| info.info.begin);

        Unicorn::mem_unmap(self, addr, size)
    }
    fn is_mapped(&self, addr: u64, size: usize) -> bool {
        let mut begin = addr;
        let mut end = addr + size as u64;
        for i in &self.get_data().memories.map_info {
            if begin < i.info.begin {
                break;
            }
            if end <= i.info.end {
                return true;
            }
            begin = i.info.end
        }
        false
    }

    fn next_mmap_address(&self, base_address: u64, size: usize) -> Result<u64, EmulatorError> {
        let mut addr = base_address;
        let size = size as u64;
        for i in &self.get_data().memories.map_info {
            if i.info.begin < base_address {
                continue;
            }
            if addr + size <= i.info.begin {
                break;
            }
            addr = i.info.end
        }
        if addr + size > (1 << 32) - 1 {
            return Err(from_raw_syscall_ret(-12)); // ENOMEM, Cannot allocate memory
        }
        Ok(addr)
    }

    fn mprotect(&mut self, addr: u64, size: usize, perm: Permission) -> Result<(), uc_error> {
        // TODO: manage map_info
        Unicorn::mem_protect(self, addr, size, perm)
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
