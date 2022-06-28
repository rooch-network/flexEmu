use crate::memory::MemoryManager;
use crate::registers::RegisterInfo;

pub struct Data {
    pub(crate) register_info: RegisterInfo,
    pub(crate) memories: MemoryManager,
}
