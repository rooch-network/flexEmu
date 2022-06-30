use crate::arch::{ArchT, Packer, Stack};
use crate::errors::EmulatorError;
use crate::memory::Memory;
use crate::registers::Registers;
use crate::utils::{align, align_up, seg_perm_to_uc_prot};
use crate::{errors, PAGE_SIZE};
use anyhow::{anyhow, bail};
use byteorder::BigEndian;
use bytes::{BufMut, BytesMut};
use goblin::container::Endian;
use goblin::elf::program_header::PT_LOAD;
use goblin::elf::Elf;
use log::debug;
use unicorn_engine::unicorn_const::{uc_error, MemRegion, Permission};

pub struct Config {
    stack_address: u64,
    stack_size: u64,
    load_address: u64,
    mmap_address: u64,
}

pub struct ElfLoader {
    config: Config,
    stack_address: u32,
    entrypoint: u32,
    elf_mem_start: u32,
    elf_entry: u32,
}
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct LoadResult {
    entrypoint: u64,
    elf_mem_start: u64,
    elf_entry: u64,

    brk_address: u64,
    mmap_address: u64,
    load_address: u64,
    init_stack_address: u64,
}

impl ElfLoader {
    pub fn load(
        config: &Config,
        binary: impl AsRef<[u8]>,
        argv: Vec<String>,
        uc: &mut (impl Memory + Registers + Stack + ArchT),
    ) -> Result<LoadResult, errors::EmulatorError> {
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

        let mut load_result = LoadResult::default();

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
        uc.set_sp(stack_address)?;
        // set elf table
        Self::load_elf_table(uc, argv)?;
        Ok(load_result)
    }
    fn load_elf_table(
        uc: &mut (impl Memory + Registers + Stack + ArchT),
        argv: Vec<String>,
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
        // TODO: fill me
        // add a nullptr sentinel
        elf_table.put_slice(&packer.pack(0));

        // TODO: rewrite to stack_write_bytes
        // write elf table
        {
            let new_stack_addr = align((uc.sp()? as usize - elf_table.len()) as u32, 0x10) as u64;
            Memory::write(uc, new_stack_addr, elf_table.as_ref())?;
            uc.set_sp(new_stack_addr)?;
        }

        Ok(())
    }

    fn load_elf_segments(
        uc: &mut (impl Memory),
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
                    Err(goblin::error::Error::Malformed(format!(
                        "invalid elf file, segment intersect."
                    )))?;
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
