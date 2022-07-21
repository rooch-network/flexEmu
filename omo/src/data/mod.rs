use crate::arch::ArchT;
use crate::config::OmoConfig;
use crate::emulator::Emu;
use crate::errors::EmulatorError;
use crate::loader::{ElfLoader, LoadInfo};
use crate::memory::MemoryManager;
use crate::os::Os;
use crate::registers::RegisterInfo;
use unicorn_engine::Unicorn;

#[derive(Debug)]
pub struct EngineData<A, O> {
    pub(crate) register_info: RegisterInfo,
    pub(crate) memories: MemoryManager,
    pub(crate) arch_info: A,
    os: O,
}

impl<A, O> EngineData<A, O> {
    pub fn os(&self) -> &O {
        &self.os
    }
    pub fn os_mut(&mut self) -> &mut O {
        &mut self.os
    }
}

impl<A, O> EngineData<A, O>
where
    A: ArchT,
    O: Os,
{
    pub fn new(arch: A, os: O) -> Self {
        let data = EngineData {
            register_info: RegisterInfo::new(arch.pc_reg_id(), arch.sp_reg_id()),
            memories: MemoryManager::default(),
            arch_info: arch,
            os,
        };
        data
    }

    pub fn build<'a>(self) -> Unicorn<'a, EngineData<A, O>> {
        let data = self;
        let uc =
            Unicorn::new_with_data(data.arch_info.arch(), data.arch_info.mode(), data).unwrap();
        uc
    }

    pub fn load<'a>(
        mut self,
        config: OmoConfig,
        binary: impl AsRef<[u8]>,
        argv: Vec<String>,
    ) -> Result<Emu<'a, A, O>, EmulatorError> {
        let mut uc = self.build();
        let binary = binary.as_ref();
        let load_result = ElfLoader::load(&config.os, binary, argv, &mut uc)?;
        O::on_load(&mut uc, load_result.clone())?;
        Ok(Emu {
            core: uc,
            loader_info: load_result,
        })
    }
}
