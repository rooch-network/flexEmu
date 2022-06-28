use crate::arch::Core;
use crate::registers::Registers;
use std::collections::{BTreeMap, HashMap};
use unicorn_engine::unicorn_const::Arch::MIPS;
use unicorn_engine::unicorn_const::{uc_error, Arch};
use unicorn_engine::{
    RegisterARM, RegisterARM64, RegisterMIPS, RegisterPPC, RegisterRISCV, RegisterX86,
};

pub enum SysCalls {
    WRITE,
    GETPID,
}

pub struct OsLinux {
    syscall_table: BTreeMap<Arch, BTreeMap<u64, SysCalls>>,
}

impl OsLinux {
    pub fn load<'a>(&self, arch: &mut Core<'a>) {
        let intr_signal = match arch.arch() {
            MIPS => 7,
            _ => unimplemented!(),
        };
        arch.uc_mut().add_intr_hook({
            move |uc, signal| {
                if signal != intr_signal {
                    return;
                }
                let syscall = get_syscall(uc.get_arch(), uc).unwrap();
            }
        });
    }
}
fn syscall_id_reg(arch: Arch) -> i32 {
    match arch {
        Arch::MIPS => RegisterMIPS::V0 as i32,
        Arch::ARM => RegisterARM::R7 as i32,
        Arch::ARM64 => RegisterARM64::X8 as i32,
        Arch::X86 => RegisterX86::EAX as i32,
        Arch::RISCV => RegisterRISCV::A7 as i32,
        _ => unimplemented!(),
    }
}

fn get_syscall(arch: Arch, registers: &impl Registers) -> Result<u64, uc_error> {
    registers.read(syscall_id_reg(arch))
}
// pub trait SysCall {
//     const NUM: u64;
//     const A: usize;
//     fn call<'a>(&self, arch: &mut Core<'a>, params: &[u64; Self::A]) -> Option<u64>;
// }
//
// pub struct SysCallWrite;
//
// impl SysCall for SysCallWrite {
//     const NUM: u64 = 1;
//     const A: usize = 10;
//     fn call<'a>(&self, arch: &mut Core<'a>, params: &[u64; Self::A]) -> Option<u64> {
//         todo!()
//     }
// }
