use crate::arch::{ArchT, MIPS};
use crate::cc::CallingConvention;
use crate::core::Core;
use crate::errors::EmulatorError;
use crate::loader::LoadInfo;
use crate::memory::Memory;
use crate::os::linux::syscall::SysCalls;
use crate::os::Os;
use crate::registers::Registers;
use crate::registers::StackRegister;
use crate::utils::{align_up, Packer};
use std::collections::HashMap;
use std::io::{stderr, stdin, stdout, Write};
use std::marker::PhantomData;
use unicorn_engine::unicorn_const::{uc_error, Arch, MemRegion, Permission};
use unicorn_engine::{
    RegisterARM, RegisterARM64, RegisterMIPS, RegisterRISCV, RegisterX86, Unicorn,
};

pub mod syscall;

#[derive(Debug, Default)]
pub struct Linux<H> {
    sigaction_act: HashMap<u64, Vec<u64>>,
    brk_address: u64,
    handler: H,
}

impl<H> Os for Linux<H>
where
    H: LinuxSyscallHandler,
{
    fn on_load<A: ArchT>(
        core: &mut Core<A, Self>,
        load_info: LoadInfo,
    ) -> Result<(), EmulatorError> {
        let os = core.get_data_mut().os_mut();
        os.brk_address = load_info.brk_address;

        core.add_intr_hook(Self::on_interrupt::<A>);
        Ok(())
    }
}

impl<H> Linux<H>
where
    H: LinuxSyscallHandler,
{
    fn on_interrupt<A: ArchT>(core: &mut Core<A, Self>, signal: u32) {
        core.get_arch()
        let signal = intr_signal(core.get_arch());
        if signal != intr_signal {
            return;
        }
        let arch = core.get_arch();
        let syscall_no = get_syscall(arch, core).unwrap();
        let call = syscall::SYSCALL
            .get(&(core.get_arch() as u8))
            .and_then(|v| v.get(&syscall_no));
        match call {
            None => {
                unimplemented!("Please implement syscall {} for {}", syscall_no, arch);
            }
            Some(call) => match SysCalls::from_str(call.as_str()) {
                Ok(c) => {
                    Self::handle_syscall(core, c).unwrap();
                }
                Err(e) => {
                    unimplemented!("Please implement syscall {} for {}", syscall_no, arch);
                }
            },
        }
    }

    fn handle_syscall<A: ArchT>(
        core: &mut Core<A, Self>,
        syscall: SysCalls,
    ) -> Result<(), EmulatorError> {
        let retvalue = match syscall {
            SysCalls::SET_THREAD_AREA => {
                let p0 = core.get_raw_param(0, None)?;
                H::set_thread_area(core, p0)?
            }
            SysCalls::SET_TID_ADDRESS => {
                let p0 = core.get_raw_param(0, None)?;
                H::set_tid_address(core, p0)?
            }
            SysCalls::POLL => {
                let p0 = core.get_raw_param(0, None)?;
                let p1 = core.get_raw_param(1, None)?;
                let p2 = core.get_raw_param(2, None)?;
                H::poll(core, p0, p1, p2)?
            }
            SysCalls::RT_SIGACTION => {
                let p0 = core.get_raw_param(0, None)?;
                let p1 = core.get_raw_param(1, None)?;
                let p2 = core.get_raw_param(2, None)?;
                H::rt_sigaction(core, p0, p1, p2)?
            }
            SysCalls::RT_SIGPROCMASK => {
                let p0 = core.get_raw_param(0, None)?;
                let p1 = core.get_raw_param(1, None)?;
                let p2 = core.get_raw_param(2, None)?;
                let p3 = core.get_raw_param(3, None)?;
                H::rt_sigprocmask(core, p0, p1, p2, p3)?
            }
            SysCalls::SIGALTSTACK => {
                let p0 = core.get_raw_param(0, None)?;
                let p1 = core.get_raw_param(1, None)?;
                H::sigaltstack(core, p0, p1)?
            }
            SysCalls::BRK => {
                let p0 = core.get_raw_param(0, None)?;
                H::brk(core, p0)?
            }
            SysCalls::WRITE => {
                // {"fd": 1, "buf": 4599872, "count": 12}
                let p0 = core.get_raw_param(0, None)?;
                let p1 = core.get_raw_param(1, None)?;
                let p2 = core.get_raw_param(2, None)?;

                H::write(core, p0, p1, p2)?
            }
            SysCalls::EXIT_GROUP => {
                let p0 = core.get_raw_param(0, None)?;
                H::exit_group(core, p0)?
            }
            _ => {
                panic!("please handle syscall: {:?}", syscall);
            }
        };
        core.set_return_value(retvalue as u64)?;
        Ok(())
    }
}

pub trait LinuxSyscallHandler: Sized {
    fn set_thread_area<A: ArchT>(
        core: &mut Core<A, Linux<Self>>,
        u_info_addr: u64,
    ) -> Result<i64, uc_error>;
    fn set_tid_address<A: ArchT>(core: &mut Core<A, Self>, tidptr: u64) -> Result<i64, uc_error>;
    fn poll<A: ArchT>(
        core: &mut Core<A, Self>,
        fds: u64,
        nfds: u64,
        timeout: u64,
    ) -> Result<i64, uc_error>;
    fn rt_sigaction<A: ArchT>(
        core: &mut Core<A, Self>,
        signum: u64,
        act: u64,
        oldact: u64,
    ) -> Result<i64, uc_error>;
    fn rt_sigprocmask<A: ArchT>(
        core: &mut Core<A, Self>,
        how: u64,
        nset: u64,
        oset: u64,
        sigsetsize: u64,
    ) -> Result<i64, uc_error>;
    fn syscall_signal(
        core: &mut Core<MIPS, Self>,
        sig: u64,
        sighandler: u64,
    ) -> Result<i64, uc_error>;
    fn sigaltstack<A: ArchT>(core: &mut Core<A, Self>, ss: u64, oss: u64) -> Result<i64, uc_error>;
    fn brk<A: ArchT>(core: &mut Core<A, Self>, inp: u64) -> Result<i64, uc_error>;
    fn write<A: ArchT>(
        core: &mut Core<A, Self>,
        fd: u64,
        buf: u64,
        count: u64,
    ) -> Result<i64, uc_error>;
    fn exit_group<A: ArchT>(core: &mut Core<A, Self>, code: u64) -> Result<i64, uc_error>;
}

pub struct DefaultLinuxSyscallHandler;

impl LinuxSyscallHandler for DefaultLinuxSyscallHandler {
    fn set_thread_area<A: ArchT>(
        core: &mut Core<A, Self>,
        u_info_addr: u64,
    ) -> Result<i64, uc_error> {
        const CONFIG4_ULR: u64 = 1 << 13;
        core.reg_write(RegisterMIPS::CP0_CONFIG3, CONFIG4_ULR)?;
        core.reg_write(RegisterMIPS::CP0_USERLOCAL, u_info_addr)?;
        core.reg_write(RegisterMIPS::V0, 0)?;
        core.reg_write(RegisterMIPS::A3, 0)?;
        log::debug!("set_thread_area({})", u_info_addr);
        Ok(0)
    }
    fn set_tid_address<A: ArchT>(core: &mut Core<A, Self>, tidptr: u64) -> Result<i64, uc_error> {
        // TODO: check thread management
        log::debug!("set_tid_address({})", tidptr);
        return Ok(std::process::id() as i64);
    }
    fn poll<A: ArchT>(
        core: &mut Core<A, Self>,
        fds: u64,
        nfds: u64,
        timeout: u64,
    ) -> Result<i64, uc_error> {
        log::debug!(
            "poll({}, {}, {}), pc: {}, sp: {}",
            fds,
            nfds,
            timeout,
            core.pc()?,
            core.sp()?
        );
        Ok(0)
    }

    fn rt_sigaction<A: ArchT>(
        core: &mut Core<A, Self>,
        signum: u64,
        act: u64,
        oldact: u64,
    ) -> Result<i64, uc_error> {
        if oldact != 0 {
            let arr = core
                .get_data()
                .arch_info
                .sigaction_act
                .get(&signum)
                .map(|s| s.as_slice())
                .unwrap_or(&[0u64; 5]);

            let data = {
                let packer = Packer::new(core.endian(), 4);
                arr.iter()
                    .map(|v| packer.pack(*v as u64))
                    .fold(vec![], |mut acc, mut v| {
                        acc.append(&mut v);
                        acc
                    })
            };
            Memory::write(core, oldact, data.as_slice())?;
        }
        if act != 0 {
            let data = (0..5)
                .map(|i| Memory::read_ptr(core, act + i * 4, Some(4)))
                .collect::<Result<Vec<_>, _>>()?;
            core.get_data_mut()
                .arch_info
                .sigaction_act
                .insert(signum, data);
        }

        log::debug!(
            "rt_sigaction({}, {}, {}), pc: {}",
            signum,
            act,
            oldact,
            core.pc()?
        );
        Ok(0)
    }
    fn rt_sigprocmask<A: ArchT>(
        core: &mut Core<A, Self>,
        how: u64,
        nset: u64,
        oset: u64,
        sigsetsize: u64,
    ) -> Result<i64, uc_error> {
        log::debug!(
            "rt_sigprocmask({}, {}, {}, {}), pc: {}",
            how,
            nset,
            oset,
            sigsetsize,
            core.pc()?
        );
        Ok(0)
    }
    fn syscall_signal(
        core: &mut Core<MIPS, Self>,
        sig: u64,
        sighandler: u64,
    ) -> Result<i64, uc_error> {
        Ok(0)
    }

    // TODO: not implemented .
    fn sigaltstack<A: ArchT>(core: &mut Core<A, Self>, ss: u64, oss: u64) -> Result<i64, uc_error> {
        log::warn!(
            "not implemented, sigaltstack({}, {}) pc: {}",
            ss,
            oss,
            core.pc()?
        );
        Ok(0)
    }
    fn brk<A: ArchT>(core: &mut Core<A, Self>, inp: u64) -> Result<i64, uc_error> {
        log::debug!("brk({}) pc: {}", inp, core.pc()?);
        // current brk_address will be modified if inp is not NULL(zero)
        // otherwise, just return current brk_address
        if inp != 0 {
            let cur_brk_addr = core.get_data().load_info.unwrap().brk_address;
            let new_brk_addr = align_up(inp as u32, core.pagesize() as u32) as u64;
            if inp > cur_brk_addr {
                Memory::mem_map(
                    core,
                    MemRegion {
                        begin: cur_brk_addr,
                        end: new_brk_addr,
                        perms: Permission::ALL,
                    },
                    Some("[brk]".to_string()),
                )?;
            } else if inp < cur_brk_addr {
                Memory::mem_unmap(core, new_brk_addr, (cur_brk_addr - new_brk_addr) as usize)?;
            }
            core.get_data_mut().load_info.as_mut().unwrap().brk_address = new_brk_addr;
        }
        Ok(core.get_data().load_info.unwrap().brk_address as i64)
    }

    fn write<A: ArchT>(
        core: &mut Core<A, Self>,
        fd: u64,
        buf: u64,
        count: u64,
    ) -> Result<i64, uc_error> {
        log::debug!("write({}, {}, {}) pc: {}", fd, buf, count, core.pc()?);
        const NR_OPEN: u64 = 1024;
        if fd > 1024 {
            return Ok(-(EBADF as i64));
        }
        let data = match Memory::read(core, buf, count as usize) {
            Ok(d) => d,
            Err(e) => {
                return Ok(-1);
            }
        };
        if fd == 1 {
            stdout().write_all(data.as_slice());
        } else if fd == 2 {
            stderr().write_all(data.as_slice());
        } else {
            return Ok(-1);
        }

        Ok(count as i64)
    }
    fn exit_group<A: ArchT>(core: &mut Core<A, Self>, code: u64) -> Result<i64, uc_error> {
        log::debug!("exit_group({}) pc: {}", code, core.pc()?);
        core.emu_stop()?;
        Ok(0)
    }
}

const EBADF: u64 = 9;

#[inline]
fn intr_signal(arch: Arch) -> u32 {
    match arch {
        Arch::MIPS => 17,
        Arch::RISCV => 8,
        Arch::ARM => 2,
        Arch::ARM64 => 2,
        _ => unimplemented!(),
    }
}

#[inline]
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
    use crate::os::linux::syscall::SysCalls;

    #[test]
    fn test_syscall_serde() {
        use strum::VariantNames;
        println!("{}", serde_json::to_string(&SysCalls::_LLSEEK).unwrap());
        let s: SysCalls = serde_json::from_str("\"_llseek\"").unwrap();
        println!("{:?}", s);
        println!("{:?}", SysCalls::VARIANTS);
    }
}
