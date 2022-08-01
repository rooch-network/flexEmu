use crate::{
    arch::{ArchInfo, ArchT},
    memory::{Memory, MemoryManager},
    registers::{Registers, StackRegister},
    stack::Stack,
    utils::align,
};
use hex::ToHex;
use serde::{Deserialize, Serialize};
use std::{
    borrow::{Borrow, Cow},
    collections::{btree_map::Entry, BTreeMap},
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
};
use unicorn_engine::{unicorn_const::Mode, Unicorn};
pub type Engine<'a, A> = Unicorn<'a, Machine<A>>;

#[derive(Debug)]
pub struct Machine<A> {
    pub(crate) memories: MemoryManager,
    pub(crate) state: MachineState,
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
            state: Default::default(),
        };
        let uc = Unicorn::new_with_data(A::T, mode, data).unwrap();
        uc
    }
}

pub trait Mach: Stack + Registers + Memory + StackRegister + ArchInfo {}

impl<T> Mach for T where T: Stack + Registers + Memory + StackRegister + ArchInfo {}

#[derive(Default, Debug, Clone)]
pub struct MachineState {
    //pub steps: u64,
    pub memory: MemoryState,
}

impl MachineState {
    pub fn snapshot(&self) -> MachineState {
        let mut s = self.clone();
        s.memory.shrink();
        s
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct MemoryState {
    data: BTreeMap<u64, Chunk>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[repr(transparent)]
struct Chunk(#[serde(with = "hex")] [u8; 4]);

impl Chunk {
    fn is_zero(&self) -> bool {
        self.0.eq(&[0; 4])
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", self.0.as_slice().encode_hex::<String>())
    }
}

impl Deref for Chunk {
    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Chunk {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MemoryState {
    /// delete blank memory chunk
    fn shrink(&mut self) {
        self.data.retain(|k, v| !v.is_zero())
    }

    fn index_chunk_mut(&mut self, addr: u64) -> &mut [u8] {
        let addr_start = align(addr, 4u8);

        let cur_chunk = match self.data.entry(addr_start) {
            Entry::Vacant(v) => {
                let cur_chunk = v.insert(Chunk([0; 4])).as_mut_slice();
                cur_chunk
            }
            Entry::Occupied(o) => {
                let cur_chunk = o.into_mut().as_mut_slice();
                cur_chunk
            }
        };
        &mut cur_chunk[(addr - addr_start) as usize..]
    }

    pub fn write_bytes(&mut self, mut addr: u64, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            let chunk = self.index_chunk_mut(addr);
            if bytes.len() < chunk.len() {
                let (left, _) = chunk.split_at_mut(bytes.len());
                left.copy_from_slice(bytes);
                return;
            } else {
                chunk.copy_from_slice(&bytes[0..chunk.len()]);

                addr += chunk.len() as u64;
                bytes = &bytes[chunk.len()..];
            }
        }
    }
    pub fn write_value(&mut self, addr: u64, size: usize, value: i64) {
        let value_bytes = (value as u32).to_be_bytes();
        self.write_bytes(addr, &value_bytes[(4 - size)..])
    }

    fn index_chunk(&self, addr: u64) -> Cow<[u8]> {
        let addr_start = align(addr, 4u8);
        match self.data.get(&addr_start) {
            Some(chunk) => Cow::Borrowed(&chunk.as_slice()[(addr - addr_start) as usize..]),
            None => Cow::Owned(vec![0; (4 + addr_start - addr) as usize]),
        }
    }

    pub fn read_bytes(&self, mut addr: u64, mut size: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(size);
        while size > 0 {
            let chunk = self.index_chunk(addr);

            let chunk_slice = if chunk.len() > size {
                &chunk[0..size]
            } else {
                &chunk
            };
            result.extend_from_slice(chunk_slice);
            addr += chunk_slice.len() as u64;
            size -= chunk_slice.len();
        }

        result
    }
}

#[cfg(test)]
mod test {
    use crate::engine::MemoryState;

    #[test]
    fn test_memory_state() {
        let mut state = MemoryState::default();
        state.write_value(2146684216, 4, 4772032);
        let data = state.read_bytes(2146684216, 4);
        let value = u32::from_be_bytes(data[0..4].try_into().unwrap());

        assert_eq!(value, 4772032);
    }
}
