use crate::{arch::ArchT, core::Core};

use crate::{errors::EmulatorError, loader::LoadInfo};

pub mod linux;

pub trait Runner {
    fn on_load<'a, A: ArchT>(
        &mut self,
        core: &mut Core<'a, A>,
        load_info: LoadInfo,
    ) -> Result<(), EmulatorError>;

    fn run<'a, A: ArchT>(&mut self, _core: &mut Core<'a, A>) -> Result<(), EmulatorError> {
        Ok(())
    }
}
