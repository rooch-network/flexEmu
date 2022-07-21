use crate::arch::ArchT;
use crate::cc::CallingConvention;
use crate::data::{EngineData};
use crate::memory::{Memory, MemoryManager};
use crate::registers::{RegisterInfo, Registers};
use crate::stack::Stack;

use unicorn_engine::Unicorn;
use crate::loader::LoadInfo;

pub type Core<'a, A, O> = Unicorn<'a, EngineData<A, O>>;

// pub fn build_core<'a, A: ArchT>(arch: A) -> Core<'a, A, O> {
//     let data = Data {
//         register_info: RegisterInfo::new(arch.pc_reg_id(), arch.sp_reg_id()),
//         memories: MemoryManager::default(),
//         arch_info: arch,
//         load_info: None,
//     };
//     let uc = Unicorn::new_with_data(data.arch_info.arch(), data.arch_info.mode(), data).unwrap();
//     uc
// }

// pub struct Core<'a, A> {
//     uc: Unicorn<'a, Data<A>>,
// }
//
// impl<'a, A> From<Unicorn<'a, Data<A>>> for Core<'a, A> {
//     fn from(uc: Unicorn<'a, Data<A>>) -> Self {
//         Self { uc }
//     }
// }
//
// impl<'a, A> Into<Unicorn<'a, Data<A>>> for Core<'a, A> {
//     fn into(self) -> Unicorn<'a, Data<A>> {
//         self.uc
//     }
// }
//
// impl<'a, A> Deref for Core<'a, A> {
//     type Target = Unicorn<'a, Data<A>>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.uc
//     }
// }
// impl<'a, A> DerefMut for Core<'a, A> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.uc
//     }
// }
//
// impl<'a, A: ArchT> Core<'a, A> {
//     pub fn new(arch: A) -> Self {
//         let data = Data {
//             register_info: RegisterInfo::new(arch.pc_reg_id(), arch.sp_reg_id()),
//             memories: MemoryManager::default(),
//             arch_info: arch,
//         };
//         let uc = Unicorn::new_with_data(arch.arch(), arch.mode(), data).unwrap();
//         Self { uc }
//     }
//     // pub fn registers_mut(&mut self,) -> &mut Unicorn<'a, ()> {
//     //     self.uc.get_mut()
//     // }
//     // pub fn registers(&self) -> &Unicorn<'a, ()> {
//     //     self.registers.borrow().deref()
//     // }
// }
