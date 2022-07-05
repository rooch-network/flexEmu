use crate::arch::{ArchInfo, ArchT};
use crate::memory::MemoryManager;
use crate::registers::RegisterInfo;
use goblin::container::Endian;

#[derive(Debug)]
pub struct Data<A> {
    pub(crate) register_info: RegisterInfo,
    pub(crate) memories: MemoryManager,
    pub(crate) arch_info: A,
}
