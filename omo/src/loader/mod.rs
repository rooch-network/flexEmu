use crate::{
    engine::Mach,
    errors,
    errors::EmulatorError,
    memory::Memory,
    stack::Stack,
    utils::{align, align_up, seg_perm_to_uc_prot, Packer},
    PAGE_SIZE,
};
use anyhow::anyhow;
use bytes::{BufMut, BytesMut};
use goblin::{
    container::Endian,
    elf::{program_header::PT_LOAD, Elf},
};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use unicorn_engine::unicorn_const::{uc_error, MemRegion, Permission};

/// auxiliary vector types
/// see: https://man7.org/linux/man-pages/man3/getauxval.3.html
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub enum AUXV {
    AT_NULL = 0,
    AT_IGNORE = 1,
    AT_EXECFD = 2,
    AT_PHDR = 3,
    AT_PHENT = 4,
    AT_PHNUM = 5,
    AT_PAGESZ = 6,
    AT_BASE = 7,
    AT_FLAGS = 8,
    AT_ENTRY = 9,
    AT_NOTELF = 10,
    AT_UID = 11,
    AT_EUID = 12,
    AT_GID = 13,
    AT_EGID = 14,
    AT_PLATFORM = 15,
    AT_HWCAP = 16,
    AT_CLKTCK = 17,
    AT_SECURE = 23,
    AT_BASE_PLATFORM = 24,
    AT_RANDOM = 25,
    AT_HWCAP2 = 26,
    AT_EXECFN = 31,
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Config {
    pub stack_address: u64,
    pub stack_size: u64,
    pub load_address: u64,
    pub mmap_address: u64,
}

/// Elf binary loader.
/// See [How programs get run: ELF binaries](https://lwn.net/Articles/631631/).
pub struct ElfLoader {
    config: Config,
    stack_address: u32,
    entrypoint: u32,
    elf_mem_start: u32,
    elf_entry: u32,
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct LoadInfo {
    pub entrypoint: u64,
    pub elf_mem_start: u64,
    pub elf_entry: u64,

    pub brk_address: u64,
    pub mmap_address: u64,
    pub load_address: u64,
    pub init_stack_address: u64,
}

impl ElfLoader {
    pub fn load(
        config: &Config,
        binary: impl AsRef<[u8]>,
        argv: Vec<String>,
        uc: &mut impl Mach,
    ) -> Result<LoadInfo, errors::EmulatorError> {
        let stack_address = config.stack_address;
        let stack_size = config.stack_size;

        uc.mem_map(
            MemRegion {
                begin: stack_address,
                end: stack_address + stack_size,
                perms: Permission::ALL,
            },
            Some("[stack]".to_string()),
        )?;

        let b = binary.as_ref();
        let elf = Elf::parse(b)?;

        if elf.header.e_type != goblin::elf::header::ET_EXEC {
            return Err(anyhow!("binary not exec"))?;
        }
        // anyhow::ensure!(
        //     elf.header.e_type == goblin::elf::header::ET_EXEC,
        //
        // )?;

        let load_address = 0;

        let (mem_start, mem_end) = Self::load_elf_segments(uc, b, &elf, load_address)?;
        debug!("mem_start: {}, mem_end: {}", mem_start, mem_end);

        let mut load_result = LoadInfo::default();

        let entrypoint = load_address + elf.header.e_entry;

        load_result.load_address = load_address;
        load_result.entrypoint = entrypoint;

        load_result.elf_entry = entrypoint;
        load_result.elf_mem_start = mem_start;
        // note: 0x2000 is the size of [hook_mem]
        load_result.brk_address = mem_end + 0x2000;

        // set info to be used by gdb
        load_result.mmap_address = config.mmap_address;
        load_result.init_stack_address = uc.sp()?;

        // init stack address
        uc.set_sp(stack_address + stack_size)?;
        // set elf table
        Self::load_elf_table(uc, &elf, &load_result, argv, BTreeMap::default())?;
        Ok(load_result)
    }
    fn load_elf_table(
        uc: &mut impl Mach,
        elf: &Elf,
        load_result: &LoadInfo,
        argv: Vec<String>, // argv.len must >0
        envs: BTreeMap<String, String>,
    ) -> Result<(), uc_error> {
        let packer = Packer::new(uc.endian(), uc.pointer_size());
        let mut elf_table = BytesMut::new();
        // write argc
        elf_table.put_slice(&packer.pack(argv.len() as u64));
        // write argv
        for s in &argv {
            uc.aligned_push_str(s)?;
            let stack_addr = uc.sp()?;
            elf_table.put_slice(&packer.pack(stack_addr));
        }
        // add a nullptr sentinel
        elf_table.put_slice(&packer.pack(0));

        // write env
        for (k, v) in &envs {
            uc.aligned_push_str(&format!("{}={}", k, v))?;
            elf_table.put_slice(&packer.pack(uc.sp()?));
        }
        // add a nullptr sentinel
        elf_table.put_slice(&packer.pack(0));

        let execfn = {
            uc.aligned_push_str(argv.first().map(|s| s.as_str()).unwrap_or_else(|| "main"))?;
            uc.sp()?
        };
        let randdata_addr = {
            uc.aligned_push_bytes(&[10u8; 16], None)?;
            uc.sp()?
        };
        let cpustr_addr = {
            uc.aligned_push_str(&format!("{:?}", uc.arch()))?;
            uc.sp()?
        };
        let aux_entries = vec![
            (
                AUXV::AT_HWCAP,
                if uc.pointer_size() == 8 {
                    0x078bfbfd
                } else if uc.pointer_size() == 4 {
                    if uc.endian() == Endian::Big {
                        // FIXME: considering this is a 32 bits value, it is not a big-endian version of the
                        // value above like it is meant to be, since the one above has an implied leading zero
                        // byte (i.e. 0x001fb8d7) which the EB value didn't take into account
                        0xd7b81f
                    } else {
                        0x1fb8d7
                    }
                } else {
                    unimplemented!()
                },
            ),
            (AUXV::AT_PAGESZ, uc.pagesize()),
            (AUXV::AT_CLKTCK, 100),
            // following three: store aux vector data for gdb use
            (
                AUXV::AT_PHDR,
                elf.header.e_phoff + load_result.elf_mem_start,
            ),
            (AUXV::AT_PHENT, elf.header.e_phentsize as u64),
            (AUXV::AT_PHNUM, elf.header.e_phnum as u64),
            (AUXV::AT_BASE, 0),
            (AUXV::AT_FLAGS, 0),
            (AUXV::AT_ENTRY, load_result.elf_entry),
            (AUXV::AT_UID, 1000),
            (AUXV::AT_EUID, 1000),
            (AUXV::AT_GID, 1000),
            (AUXV::AT_EGID, 1000),
            (AUXV::AT_SECURE, 0),
            (AUXV::AT_RANDOM, randdata_addr),
            (AUXV::AT_HWCAP2, 0),
            (AUXV::AT_EXECFN, execfn),
            (AUXV::AT_PLATFORM, cpustr_addr),
            (AUXV::AT_NULL, 0),
        ];
        for (k, v) in aux_entries {
            elf_table.extend_from_slice(&packer.pack(k as u64));
            elf_table.extend_from_slice(&packer.pack(v));
        }

        // write elf table
        Stack::aligned_push_bytes(uc, elf_table.as_ref(), Some(0x10))?;

        Ok(())
    }

    fn load_elf_segments(
        uc: &mut impl Memory,
        binary: impl AsRef<[u8]>,
        elf: &Elf,
        load_address: u64,
    ) -> Result<(u64, u64), EmulatorError> {
        // get list of loadable segments which will be loaded into memory.
        let load_segments = {
            let mut load_segments = elf
                .program_headers
                .iter()
                .filter(|h| h.p_type == PT_LOAD)
                .collect::<Vec<_>>();
            load_segments.sort_by_key(|p| p.p_vaddr);
            load_segments
        };

        let mut load_regions = Vec::new();
        for seg in &load_segments {
            let lbound = align((load_address + seg.p_vaddr) as u32, PAGE_SIZE) as u64;
            let ubound =
                align_up((load_address + seg.p_vaddr + seg.p_memsz) as u32, PAGE_SIZE) as u64;
            let perms = seg_perm_to_uc_prot(seg.p_flags);
            if load_regions.is_empty() {
                load_regions.push(MemRegion {
                    begin: lbound as u64,
                    end: ubound as u64,
                    perms,
                });
            } else {
                let prev_region = load_regions.last_mut().unwrap();

                if lbound > prev_region.end {
                    load_regions.push(MemRegion {
                        begin: lbound as u64,
                        end: ubound as u64,
                        perms,
                    });
                } else if lbound == prev_region.end {
                    // new region starts where the previous one ended
                    // same perms.
                    if perms == prev_region.perms {
                        prev_region.end = ubound;
                    } else {
                        // different perms. start a new one
                        load_regions.push(MemRegion {
                            begin: lbound,
                            end: ubound,
                            perms,
                        });
                    }
                } else if lbound < prev_region.end {
                    Err(goblin::error::Error::Malformed(
                        "invalid elf file, segment intersect.".to_string(),
                    ))?;
                }
            }
        }

        for region in &load_regions {
            uc.mem_map(region.clone(), None)?;
        }

        for seg in &load_segments {
            let data = &binary.as_ref()[seg.file_range()];
            Memory::write(uc, load_address + seg.p_vaddr, data)?;
        }

        let (mem_start, mem_end) = (
            load_regions.first().unwrap().begin,
            load_regions.last().unwrap().end,
        );
        Ok((mem_start, mem_end))
    }
}
