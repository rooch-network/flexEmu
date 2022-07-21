//use crate::arch::Core;

use crate::{
    arch::{ArchInfo, ArchT},
    config::OmoConfig,
    engine::{Engine, Machine},
    errors::EmulatorError,
    loader::{ElfLoader, LoadInfo},
    os::Runner,
};
use std::collections::BTreeMap;

use unicorn_engine::unicorn_const::Mode;

pub struct Emulator<'a, A, Os> {
    config: OmoConfig,
    core: Engine<'a, A>,
    os: Os,
}

impl<'a, A, O> Emulator<'a, A, O> {
    pub fn engine(&self) -> &Engine<'a, A> {
        &self.core
    }
    pub fn runner(&self) -> &O {
        &self.os
    }
}

impl<'a, A: ArchT, O: Runner> Emulator<'a, A, O> {
    pub fn new(conf: OmoConfig, arch: A, mode: Mode, os: O) -> Result<Self, EmulatorError> {
        let machine = Machine::create(arch, mode);
        // let binary = binary.as_ref();
        // let load_result = ElfLoader::load(&config.os, binary, argv, &mut machine)?;
        // os.on_load(&mut machine, load_result.clone())?;

        Ok(Self {
            config: conf,
            core: machine,
            os,
        })
    }

    pub fn load(
        &mut self,
        binary: impl AsRef<[u8]>,
        argv: Vec<String>,
        env: Vec<(String, String)>,
    ) -> Result<LoadInfo, EmulatorError> {
        let binary = binary.as_ref();
        let load_result = ElfLoader::load(
            &self.config.os,
            binary,
            argv,
            env.into_iter().collect::<BTreeMap<_, _>>(),
            &mut self.core,
        )?;
        self.os.on_load(&mut self.core, load_result)?;
        Ok(load_result)
    }

    pub fn run(
        &mut self,
        entrypoint: u64,
        exitpoint: Option<u64>,
        timeout: Option<u64>,
        count: Option<usize>,
    ) -> Result<(), EmulatorError> {
        let exitpoint = exitpoint.unwrap_or_else(|| default_exitpoint(self.core.pointer_size()));
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
