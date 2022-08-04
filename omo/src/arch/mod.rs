pub mod mips;

use crate::{cc::CallingConvention, memory::PointerSizeT};
use goblin::container::Endian;
use unicorn_engine::{
    unicorn_const::{Arch, Mode, Query},
    Unicorn,
};

pub trait ArchT {
    type CC: CallingConvention;
    const T: Arch;
    const PC: i32;
    const SP: i32;
    fn cc(&self) -> Self::CC;
    fn registers(&self) -> &[i32];
}

pub trait ArchInfo {
    fn endian(&self) -> Endian;
    fn pointer_size(&self) -> PointerSizeT;
    fn arch(&self) -> Arch;
    fn mode(&self) -> Mode;
}

impl<'a, D> ArchInfo for Unicorn<'a, D> {
    fn endian(&self) -> Endian {
        let mode = self.mode();
        if mode.contains(Mode::BIG_ENDIAN) {
            Endian::Big
        } else {
            Endian::Little
        }
    }

    fn pointer_size(&self) -> PointerSizeT {
        let mode = self.mode();
        if mode.contains(Mode::MODE_32) {
            4
        } else if mode.contains(Mode::MODE_16) {
            2
        } else if mode.contains(Mode::MODE_64) {
            8
        } else {
            unimplemented!()
        }
    }

    fn arch(&self) -> Arch {
        self.get_arch()
    }

    fn mode(&self) -> Mode {
        Mode::from_bits(self.query(Query::MODE).unwrap() as i32).unwrap()
    }
}
