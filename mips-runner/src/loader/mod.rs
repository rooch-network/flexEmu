use goblin::elf::program_header::PT_LOAD;
use unicorn_engine::unicorn_const::MemRegion;
use crate::{align, align_up, errors, PAGE_SIZE, seg_perm_to_uc_prot};
use crate::memory::MemoryManager;

pub struct Config {
    stack_address: u32,
    stack_size: u32,
    load_address: u32,
    mmap_address: u32,
}

pub struct ElfLoader {
    config: Config,
    stack_address: u32,
    entrypoint: u32,
    elf_mem_start: u32,
    elf_entry: u32,
}

impl ElfLoader {
    pub fn load<'a>(&self, binary: impl AsRef<[u8]>, memory: &mut MemoryManager<'a>) -> Result<(), errors::EmulatorError> {
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
        for seg in &load_segments {
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
                    Err(goblin::error::Error::Malformed(format!("invalid elf file, segment intersect.")))?;
                }
            }
        }

        for region in load_regions {
            memory.mem_map(region, None)?;
        }

        for seg in &load_segments {
            let data = &b[seg.file_range()];
            memory.write((load_address + seg.p_vaddr) as u64, data)?;
        }

        let (mem_start, mem_end) = (load_regions.first().unwrap().begin, load_regions.last().unwrap().end);

        let entrypoint = load_address + elf.header.e_entry;
        let elf_entry = entrypoint;

        // note: 0x2000 is the size of [hook_mem]
        let brk_address = mem_end + 0x2000;
        memory.uc.borrow_mut().reg_write(unicorn_engine::RegisterMIPS::SP, self.stack_address as u64);
        Ok(())
    }
}
