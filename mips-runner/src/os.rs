use crate::arch::Arch;

pub trait Os {
    type SysCallT: SysCall;
}

pub struct OsLinux {
}
impl OsLinux {
    pub fn load<'a>(&self, arch: &mut Arch<'a>) {

    }
}

pub trait SysCall {
    const NUM: u64;
    const A: usize;
    fn call<'a>(&self, arch: &mut Arch<'a>, params: [u64;A]) -> Option<u64>;
}

pub struct SysCallWrite;

impl SysCall for SysCallWrite {
    const NUM: u64 = 1;
    const A: usize = 10;
    fn call<'a>(&self, arch: &mut Arch<'a>, params: [u64; A]) -> Option<u64> {
        todo!()
    }
}