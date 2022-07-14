use std::io::{stderr, stdin, stdout, Write};
use crate::arch::{ArchT, MIPS};
use crate::core::Core;
use crate::data::Data;
use crate::errors::EmulatorError;
use crate::registers::Registers;
use serde::{Deserialize, Serialize};
use strum::EnumVariantNames;
use unicorn_engine::unicorn_const::{uc_error, Arch, MemRegion, Permission};
use unicorn_engine::{
    RegisterARM, RegisterARM64, RegisterMIPS, RegisterRISCV, RegisterX86, Unicorn,
};
use crate::cc::CallingConvention;
use crate::memory::Memory;
use crate::registers::StackRegister;
use crate::utils::{align_up, Packer};

pub trait Os<A> {
    fn run<'a>(self, core: &mut Core<'a, A>) -> Result<(), EmulatorError>;
}


#[derive(Debug)]
pub struct Linux {
}

impl<A: ArchT> Os<A> for Linux where LinuxHandler: SysCallHandler<A> {
    fn run<'a>(self, core: &mut Core<'a, A>, ) -> Result<(), EmulatorError> {
        attach_handler::<A, LinuxHandler>(core)?;
        core.emu_start();
        Ok(())
    }
}

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
        log::debug!("poll({}, {}, {}), pc: {}, sp: {}", fds, nfds, timeout, core.pc()?, core.sp()?);
        Ok(0)
    }

    fn rt_sigaction(core: &mut Core<MIPS>, signum: u64, act: u64, oldact: u64) -> Result<i64, uc_error> {
        if oldact != 0 {
            let arr = core.get_data().arch_info.sigaction_act.get(&signum).map(|s| s.as_slice()).unwrap_or(&[0u64;5]);

            let data = {
                let packer = Packer::new(core.endian(), 4);
                arr.iter().map(|v|packer.pack(*v as u64)).fold(vec![], |mut acc, mut v| {
                    acc.append(&mut v);
                    acc
                })
            };
            Memory::write(core, oldact, data.as_slice())?;
        }
        if act != 0 {
            let data = (0..5).map(|i| Memory::read_ptr(core, act + i * 4, Some(4))).collect::<Result<Vec<_>, _>>()?;
            core.get_data_mut().arch_info.sigaction_act.insert(signum, data);
        }

        log::debug!("rt_sigaction({}, {}, {}), pc: {}", signum, act,oldact, core.pc()?);
        Ok(0)
    }
    fn rt_sigprocmask(core: &mut Core<MIPS>, how: u64, nset: u64, oset: u64, sigsetsize: u64) -> Result<i64, uc_error> {
        log::debug!("rt_sigprocmask({}, {}, {}, {}), pc: {}", how, nset,oset,sigsetsize, core.pc()?);
        Ok(0)
    }
    fn syscall_signal(core: &mut Core<MIPS>, sig: u64, sighandler: u64) -> Result<i64, uc_error> {
        Ok(0)
    }

    // TODO: not implemented .
    fn sigaltstack(core: &mut Core<MIPS>, ss: u64, oss: u64) -> Result<i64, uc_error>{
        log::warn!("not implemented, sigaltstack({}, {}) pc: {}", ss, oss, core.pc()?);
        Ok(0)
    }
    fn brk(core: &mut Core<MIPS>, inp: u64) -> Result<i64, uc_error> {
        log::debug!("brk({}) pc: {}", inp, core.pc()?);
        // current brk_address will be modified if inp is not NULL(zero)
        // otherwise, just return current brk_address
        if inp != 0 {
            let cur_brk_addr = core.get_data().load_info.unwrap().brk_address;
            let new_brk_addr = align_up(inp as u32, core.pagesize() as u32) as u64;
            if inp > cur_brk_addr {
                Memory::mem_map(core, MemRegion {
                    begin: cur_brk_addr,
                    end: new_brk_addr,
                    perms: Permission::ALL,
                }, Some("[brk]".to_string()))?;
            } else if inp < cur_brk_addr {
                Memory::mem_unmap(core, new_brk_addr, (cur_brk_addr - new_brk_addr) as usize)?;
            }
            core.get_data_mut().load_info.as_mut().unwrap().brk_address = new_brk_addr;
        }
        Ok(core.get_data().load_info.unwrap().brk_address as i64)
    }

    fn write(core: &mut Core<MIPS>, fd: u64, buf: u64, count: u64) -> Result<i64, uc_error> {
        log::debug!("write({}, {}, {}) pc: {}", fd, buf,count, core.pc()?);
        const NR_OPEN: u64 = 1024;
        if fd > 1024 {
            return Ok(-(EBADF as i64))
        }
        let data = match Memory::read(core, buf, count as usize) {
            Ok(d) => d,
            Err(e) => {
                return Ok(-1)
            }
        };
        if fd == 1 {
            stdout().write_all(data.as_slice());
        } else if fd == 2 {
            stderr().write_all(data.as_slice());
        } else {
            return Ok(-1)
        }

        Ok(count as i64)
    }
    fn exit_group(core: &mut Core<MIPS>, code: u64) -> Result<i64, uc_error> {
        log::debug!("exit_group({}) pc: {}", code, core.pc()?);
        core.emu_stop()?;
        Ok(0)
    }
}
const EBADF:u64           = 9;

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
            4194 => {
                let p0 = core.get_raw_param(0, None)?;
                let p1 = core.get_raw_param(1, None)?;
                let p2 = core.get_raw_param(2, None)?;
                LinuxHandler::rt_sigaction(core, p0,p1,p2)?
            }
            4195 => {
                let p0 = core.get_raw_param(0, None)?;
                let p1 = core.get_raw_param(1, None)?;
                let p2 = core.get_raw_param(2, None)?;
                let p3 = core.get_raw_param(3, None)?;
                LinuxHandler::rt_sigprocmask(core, p0,p1,p2, p3)?
            }
            4206 => {
                let p0 = core.get_raw_param(0, None)?;
                let p1 = core.get_raw_param(1, None)?;
                LinuxHandler::sigaltstack(core, p0, p1)?
            }
            4045 => {
                let p0 = core.get_raw_param(0, None)?;
                LinuxHandler::brk(core, p0)?
            }
            4004 => {
                // {"fd": 1, "buf": 4599872, "count": 12}
                let p0 = core.get_raw_param(0, None)?;
                let p1 = core.get_raw_param(1, None)?;
                let p2 = core.get_raw_param(2, None)?;

                LinuxHandler::write(core, p0, p1, p2)?

            }
            4246 => {
                let p0 = core.get_raw_param(0, None)?;
                LinuxHandler::exit_group(core, p0)?
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
