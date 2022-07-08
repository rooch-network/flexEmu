use crate::loader::LoadResult;
use crate::memory::MemoryManager;
use crate::registers::RegisterInfo;

#[derive(Debug)]
pub struct Data<A> {
    pub(crate) register_info: RegisterInfo,
    pub(crate) memories: MemoryManager,
    pub(crate) arch_info: A,
    pub(crate) load_info: Option<LoadResult>,
}
