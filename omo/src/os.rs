use crate::arch::{ArchT, MIPS};
use crate::core::Core;
use crate::data::Data;
use crate::errors::EmulatorError;
use crate::registers::Registers;
use serde::{Deserialize, Serialize};
use strum::EnumVariantNames;
use unicorn_engine::unicorn_const::{uc_error, Arch};
use unicorn_engine::{
    RegisterARM, RegisterARM64, RegisterMIPS, RegisterRISCV, RegisterX86, Unicorn,
};
use crate::cc::CallingConvention;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, EnumVariantNames)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SysCalls {
    WRITE,
    GETPID,
    _LLSEEK,
}

pub trait SysCallHandler<A> {
    fn handle(core: &mut Core<A>, syscall_no: u64) -> Result<(), EmulatorError> {
        panic!("handle syscall: {}", syscall_no);
    }
}

pub struct LinuxHandler;
impl LinuxHandler {
    fn set_thread_area(core: &mut Core<MIPS>, u_info_addr: u64) -> Result<i64, uc_error> {
        const CONFIG4_ULR: u64 = 1<<13;
        core.reg_write(RegisterMIPS::CP0_CONFIG3, CONFIG4_ULR)?;
        core.reg_write(RegisterMIPS::CP0_USERLOCAL, u_info_addr)?;
        core.reg_write(RegisterMIPS::V0, 0)?;
        core.reg_write(RegisterMIPS::A3, 0)?;
        log::debug!("set_thread_area({})", u_info_addr);
        Ok(0)
    }
    fn set_tid_address(core: &mut Core<MIPS>, tidptr: u64) -> Result<i64, uc_error> {
        // TODO: check thread management
        log::debug!("set_tid_address({})", tidptr);
        return Ok(std::process::id() as i64)
    }
    fn poll(core: &mut Core<MIPS>, fds: u64, nfds: u64,timeout: u64) -> Result<i64, uc_error> {
        log::debug!("poll({}, {}, {}), pc: {}", fds, nfds, timeout, core.pc()?);
        todo!()
    }
}
impl SysCallHandler<MIPS> for LinuxHandler {
    fn handle(core: &mut Core<MIPS>, syscall_no: u64) -> Result<(), EmulatorError> {
        let retvalue = match syscall_no {
            4283 => {
                let p0 = core.get_raw_param(0, None)?;
                LinuxHandler::set_thread_area(core, p0)?
            }
            4252 => {
                let p0 = core.get_raw_param(0, None)?;
                LinuxHandler::set_tid_address(core, p0)?
            }
            4188 => {
                let p0 = core.get_raw_param(0, None)?;
                let p1 = core.get_raw_param(1, None)?;
                let p2 = core.get_raw_param(2, None)?;
                LinuxHandler::poll(core, p0, p1, p2)?
            }
            _ => {
                panic!("handle syscall: {}", syscall_no);
            }
        };
        core.set_return_value(retvalue as u64)?;
        Ok(())
    }
}

pub fn attach_handler<A, H: SysCallHandler<A>>(core: &mut Core<A>) -> Result<(), EmulatorError> {
    core.add_intr_hook({
        move |uc, signal| {
            let intr_signal = match uc.get_arch() {
                MIPS => 17,
                _ => unimplemented!(),
            };
            if signal != intr_signal {
                return;
            }
            let syscall = get_syscall(uc.get_arch(), uc).unwrap();
            H::handle(uc, syscall).unwrap();
        }
    })?;
    Ok(())
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

pub trait LinuxSysCalls<A: ArchT> {
    fn write<'a>(&self, arch: &mut Unicorn<'a, Data<A>>, fd: i32, buf: u64, count: usize) -> isize;
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
