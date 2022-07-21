use crate::arch::{ArchInfo, ArchT};

use crate::memory::{Memory, MemoryManager};

use crate::{
    registers::{Registers, StackRegister},
    stack::Stack,
};
use unicorn_engine::{unicorn_const::Mode, Unicorn};

#[derive(Debug)]
pub struct Machine<A> {
    pub(crate) memories: MemoryManager,
    arch: A,
}

impl<A> Machine<A> {
    pub fn env(&self) -> &A {
        &self.arch
    }
}

impl<A: ArchT> Machine<A> {
    pub fn create<'a>(at: A, mode: Mode) -> Unicorn<'a, Self> {
        let data = Machine {
            memories: MemoryManager::default(),
            arch: at,
        };
        let uc = Unicorn::new_with_data(A::T, mode, data).unwrap();
        uc
    }
}

pub trait Mach: Stack + Registers + Memory + StackRegister + ArchInfo {}

impl<T> Mach for T where T: Stack + Registers + Memory + StackRegister + ArchInfo {}
