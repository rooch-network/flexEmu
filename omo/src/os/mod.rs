use crate::arch::ArchT;
use crate::core::Core;
use crate::errors::EmulatorError;
use crate::loader::LoadInfo;

pub mod linux;

pub trait Os {
    fn on_load<A: ArchT>(core: &mut Core<A, Self>, load_info: LoadInfo) -> Result<(), EmulatorError>
    where
        Self: Sized,
    {
        Ok(())
    }
}
