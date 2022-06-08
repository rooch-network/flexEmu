use std::cell::{Cell, RefCell};
use std::ops::BitOr;
use std::rc::Rc;
use anyhow::{bail, Result};
use goblin::elf::program_header::{PF_R, PF_W, PF_X, PT_LOAD};
use unicorn_engine::Unicorn;
use unicorn_engine::unicorn_const::{Arch, MemRegion, Mode, Permission};

pub struct Emulator<'a, Loader, Os> {
    mem: MemoryManager<'a>,
    loader: Loader,
    os: Os,
}

struct MapInfo {

}
pub struct MemoryManager<'a> {
    uc: Rc<RefCell<Unicorn<'a, ()>>>,
    map_info: Vec<MapInfo>
}
impl<'a> MemoryManager<'a> {
    pub fn new(uc: Rc<RefCell<Unicorn<'a, ()>>>)-> Self {
        Self {
            uc,
            map_info: Vec::new()
        }
    }
}


pub const PAGE_SIZE: u32 = 0x1000;

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
pub fn align(value: u32, alignment: u32) -> u32 {
    debug_assert_eq!(alignment & (alignment -1), 0);
    // round down to nearest alignment
    value & (!(alignment - 1))
}

pub fn align_up(value: u32, alignment: u32) -> u32 {

    debug_assert_eq!(alignment & (alignment -1), 0);
    // round up to nearest alignment
    (value + alignment - 1) & (!(alignment - 1))
}

/// Translate ELF segment perms to Unicorn protection constants.
pub fn seg_perm_to_uc_prot(perm: u32) -> Permission {
    let mut prot = Permission::NONE;
    if perm & PF_X {
        prot |= Permission::EXEC;
    }
    if perm & PF_W {
        prot |= Permission::WRITE;
    }
    if perm & PF_R {
        prot |= Permission::READ;
    }

    prot
}
pub struct Config {
    stack_address: u32,
    stack_size: u32,
    load_address: u32,
}

pub struct ElfLoader {
    config: Config,
}

impl ElfLoader {
    pub fn load(&self, binary: impl AsRef<[u8]>) -> Result<()> {
        let b= binary.as_ref();
        let elf = goblin::elf::Elf::parse(b)?;

        anyhow::ensure!(elf.header.e_type == goblin::elf::header::ET_EXEC, "binary not exec");

        // get list of loadable segments which will be loaded into memory.
        let load_segments = {
            let mut load_segments = elf.program_headers.iter().filter(|h| h.p_type == PT_LOAD).collect::<Vec<_>>();
            load_segments.sort_by_key(|p| p.p_vaddr);
            load_segments
        };
        let load_address = self.config.load_address;
        let mut load_regions = Vec::new();
        let unicorn  = unicorn_engine::Unicorn::new(Arch::MIPS, Mode::MIPS32|Mode::BIG_ENDIAN).unwrap();
        for seg in load_segments {
            let lbound = align(load_address + seg.p_vaddr, PAGE_SIZE) as u64;
            let ubound = align_up(load_address + seg.p_vaddr + seg.p_memsz, PAGE_SIZE) as u64;
            let perms = seg_perm_to_uc_prot(seg.p_flags);
            if load_regions.is_empty() {
                load_regions.push(MemRegion {
                    begin: lbound as u64,
                    end: ubound as u64,
                    perms
                });
            } else {
                let prev_region = load_regions.last_mut().unwrap();

                if lbound > prev_region.end {
                    load_regions.push(MemRegion {
                        begin: lbound as u64,
                        end: ubound as u64,
                        perms
                    });
                } else if lbound == prev_region.end { // new region starts where the previous one ended
                    // same perms.
                    if perms == prev_region.perms {
                        prev_region.end = ubound;
                    } else {
                        // different perms. start a new one
                        load_regions.push(MemRegion {
                            begin: lbound, end: ubound, perms
                        });
                    }
                } else if lbound < prev_region.end {
                    bail!("invalid elf file");
                }
            }
        }

        Ok(())
    }
}

pub struct LinuxOs {

}



#[cfg(test)]
mod test {
    use crate::{align, align_up};

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
