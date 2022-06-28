use crate::arch::Core;
use unicorn_engine::unicorn_const::Arch::MIPS;

pub trait Os {
    type SysCallT: SysCall;
}

pub struct OsLinux {}
impl OsLinux {
    pub fn load<'a>(&self, arch: &mut Core<'a>) {
        let intr_signal = match arch.arch() {
            MIPS => 7,
            _ => unimplemented!(),
        };
        arch.uc().add_intr_hook({
            |uc, signal| {
                if signal != intr_signal {
                    return;
                }
            }
        });
    }
}

pub trait SysCall {
    const NUM: u64;
    const A: usize;
    fn call<'a>(&self, arch: &mut Core<'a>, params: [u64; Self::A]) -> Option<u64>;
}

pub struct SysCallWrite;

impl SysCall for SysCallWrite {
    const NUM: u64 = 1;
    const A: usize = 10;
    fn call<'a>(&self, arch: &mut Core<'a>, params: [u64; Self::A]) -> Option<u64> {
        todo!()
    }
}
