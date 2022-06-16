use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::BitOr;
use std::rc::Rc;
use anyhow::{bail, Result};
use goblin::elf::program_header::{PF_R, PF_W, PF_X, PT_LOAD};
use unicorn_engine::{RegisterMIPS, Unicorn};
use unicorn_engine::unicorn_const::{Arch, MemRegion, Mode, Permission, uc_error};
use crate::registers::RegisterManager;

struct MapInfo {
    info: MemRegion,
    label: String,
}

pub struct MemoryManager<'a> {
    pub(crate) uc: Rc<RefCell<Unicorn<'a, ()>>>,
    map_info: Vec<MapInfo>
}
impl<'a> MemoryManager<'a> {
    pub fn new(uc: Rc<RefCell<Unicorn<'a, ()>>>)-> Self {
        Self {
            uc,
            map_info: Vec::new()
        }
    }
    pub fn mem_map(&mut self, MemRegion {begin,end,perms}: MemRegion, info: Option<String>) -> Result<(), uc_error> {
        debug_assert!(perm & (!Permission::ALL) == 0, "unexcepted permissions mask {}", perms);

        self.uc.borrow_mut().mem_map(begin, (end - begin) as usize, perms)?;
        self.add_mapinfo(MemRegion {begin,end,perms}, info.unwrap_or("[mapped]".to_string()));
        Ok(())
    }
    pub fn write(&mut self,address: u64, bytes: impl AsRef<[u8]>) -> Result<(), uc_error> {
        self.uc.borrow_mut().mem_write(address, bytes.as_ref())
    }

    fn add_mapinfo(&mut self, mem_info: MemRegion, label: String) {
        self.map_info.push(MapInfo {info: mem_info, label});
        self.map_info.sort_by_key(|info| info.info.begin);
    }
}
