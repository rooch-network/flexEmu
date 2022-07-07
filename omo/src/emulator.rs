//use crate::arch::Core;

use crate::arch::ArchT;
use crate::config::OmoConfig;
use crate::core::{build_core, Core};
use crate::errors::EmulatorError;
use crate::loader::{ElfLoader, LoadResult};
use crate::os::{attach_handler, SysCallHandler};
use std::marker::PhantomData;

pub struct Emulator<'a, A, Os> {
    arch: Core<'a, A>,
    loader_info: LoadResult,
    phantom: PhantomData<Os>,
}

impl<'a, A: ArchT, Os: SysCallHandler<A>> Emulator<'a, A, Os> {
    pub fn new(
        config: OmoConfig,
        arch: A,
        binary: impl AsRef<[u8]>,
        argv: Vec<String>,
    ) -> Result<Self, EmulatorError> {
        let mut uc = build_core(arch);
        let binary = binary.as_ref();
        let load_result = ElfLoader::load(&config.os, binary, argv, &mut uc)?;

        attach_handler::<_, Os>(&mut uc)?;

        Ok(Self {
            arch: uc,
            loader_info: load_result,
            phantom: PhantomData,
        })
    }

    pub fn run(
        &mut self,
        entrypoint: Option<u64>,
        exitpoint: Option<u64>,
        timeout: Option<u64>,
        count: Option<usize>,
    ) -> Result<(), EmulatorError> {
        let entrypoint = entrypoint.unwrap_or(self.loader_info.entrypoint);
        let exitpoint = exitpoint.unwrap_or_else(|| default_exitpoint(self.arch.pointer_size()));
        self.arch.emu_start(
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
