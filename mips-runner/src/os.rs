use crate::arch::Core;
use crate::registers::Registers;
use maplit::btreemap;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use unicorn_engine::unicorn_const::Arch::MIPS;
use unicorn_engine::unicorn_const::{uc_error, Arch};
use unicorn_engine::{
    RegisterARM, RegisterARM64, RegisterMIPS, RegisterPPC, RegisterRISCV, RegisterX86, Unicorn,
};

use crate::data::Data;
use serde::{Deserialize, Serialize};
use strum::EnumVariantNames;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, EnumVariantNames)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SysCalls {
    WRITE,
    GETPID,
    _LLSEEK,
}

#[derive(Clone)]
pub struct OsLinux {
    inner: Arc<OsLinuxInner>,
}

struct OsLinuxInner {
    syscall_table: HashMap<u8, BTreeMap<u64, SysCalls>>,
    syscalls: HashMap<SysCalls, Box<dyn SysCallT<4>>>,
}
impl OsLinux {
    pub fn load<'a>(&self, arch: &mut Core<'a>) -> Result<(), uc_error> {
        arch.uc_mut().add_intr_hook({
            let this = self.clone();
            move |uc, signal| {
                this.syscall_hook(uc, signal);
            }
        })?;
        Ok(())
    }

    fn syscall_hook(&self, uc: &mut Unicorn<Data>, signal: u32) {
        let intr_signal = match uc.get_arch() {
            MIPS => 17,
            _ => unimplemented!(),
        };
        if signal != intr_signal {
            return;
        }
        let syscall = get_syscall(uc.get_arch(), uc).unwrap();
        let ar = uc.get_arch() as u8;
        let syscall = self
            .inner
            .syscall_table
            .get(&ar)
            .and_then(|m| m.get(&syscall))
            .cloned();
        if let Some(call) = syscall {
            let handler = self.inner.syscalls.get(&call);

            if let Some(h) = handler {
                //h.call(uc);
            }
        }
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

pub trait SysCallT<const A: usize> {
    //const T: SysCalls;
    // const A: usize;
    fn call<'a>(&self, arch: &mut Unicorn<'a, Data>, params: &[u64; A]) -> i64;
}

pub trait LinuxSysCalls {
    fn write<'a>(&self, arch: &mut Unicorn<'a, Data>, fd: i32, buf: u64, count: usize) -> isize;
}
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

#[cfg(test)]
mod tests {
    use crate::os::SysCalls;

    #[test]
    fn test_syscall_serde() {
        use strum::VariantNames;
        println!("{}", serde_json::to_string(&SysCalls::_LLSEEK).unwrap());
        let s: SysCalls = serde_json::from_str("\"_llseek\"").unwrap();
        println!("{:?}", s);
        println!("{:?}", SysCalls::VARIANTS);
    }
}
