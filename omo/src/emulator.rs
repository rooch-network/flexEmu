//use crate::arch::Core;

use crate::arch::ArchT;
use crate::config::OmoConfig;
use crate::core::Core;
use crate::errors::EmulatorError;
use crate::loader::{ElfLoader, LoadInfo};
use crate::os::Os;
use std::marker::PhantomData;

pub struct Emu<'a, A, Os> {
    pub(crate) core: Core<'a, A, Os>,
    pub(crate) loader_info: LoadInfo,
}

impl<'a, A: ArchT, O: Os> Emu<'a, A, O> {
    pub fn run(
        &mut self,
        entrypoint: Option<u64>,
        exitpoint: Option<u64>,
        timeout: Option<u64>,
        count: Option<usize>,
    ) -> Result<(), EmulatorError> {
        let entrypoint = entrypoint.unwrap_or(self.loader_info.entrypoint);
        let exitpoint = exitpoint.unwrap_or_else(|| default_exitpoint(self.arch.pointer_size()));
        self.core.emu_start(
            entrypoint,
            exitpoint,
            timeout.unwrap_or_default(),
            count.unwrap_or_default(),
        )?;
        Ok(())
    }
}

pub fn default_exitpoint(point_size: u8) -> u64 {
    match point_size {
        2 => 0xfffff, // 20bit address lane
        4 => 0x8fffffff,
        8 => 0xffffffffffffffff,
        _ => unreachable!(),
    }
}
