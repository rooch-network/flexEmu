use crate::{arch::ArchT, engine::Engine};

use crate::{errors::EmulatorError, loader::LoadInfo};

pub mod linux;

pub trait Runner {
    fn on_load<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        load_info: LoadInfo,
    ) -> Result<(), EmulatorError>;

    fn run<'a, A: ArchT>(&mut self, _core: &mut Engine<'a, A>) -> Result<(), EmulatorError> {
        Ok(())
    }
}
