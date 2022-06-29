use crate::arch::ArchInfo;
use crate::memory::MemoryManager;
use crate::registers::RegisterInfo;
use goblin::container::Endian;

pub struct Data {
    pub(crate) register_info: RegisterInfo,
    pub(crate) memories: MemoryManager,
    pub(crate) arch_info: ArchInfo,
}

impl Data {
    pub fn pointersize(&self) -> u64 {
        self.arch_info.bit / 8
    }
    pub fn endian(&self) -> Endian {
        self.arch_info.endian
    }
}
