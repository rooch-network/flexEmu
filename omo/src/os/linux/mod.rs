use std::{cell::RefCell, collections::HashMap, rc::Rc, str::FromStr};

use unicorn_engine::{
    RegisterARM,
    RegisterARM64, RegisterMIPS, RegisterRISCV, RegisterX86, unicorn_const::{Arch, MemRegion, Permission, uc_error},
};

use file::{open, read, write};

use crate::{
    arch::{ArchInfo, ArchT},
    cc::CallingConvention,
    engine::Engine,
    errors::EmulatorError,
    loader::LoadInfo,
    memory::Memory,
    os::{
        linux::{
            file::{close, fcntl, lseek},
            syscall::{Rlimit, SysCalls, SysInfo},
        },
        Runner,
    },
    rand::{RAND_SOURCE, RAND_SOURCE_LEN},
    registers::{Registers, StackRegister},
    utils::{align, align_up, Packer, read_string},
};

mod file;
pub mod syscall;

#[derive(Debug, Default)]
pub struct LinuxRunner {
    inner: Rc<RefCell<Inner>>,
}

#[derive(Debug, Default)]
struct Inner {
    sigaction_act: HashMap<u64, Vec<u64>>,
    mmap_address: u64,
    brk_address: u64,
}

impl LinuxRunner {
    pub fn new(mmap_address: u64) -> Self {
        let inner = Inner {
            sigaction_act: HashMap::default(),
            mmap_address,
            brk_address: 0,
        };
        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }
}

impl Runner for LinuxRunner {
    fn on_load<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        load_info: LoadInfo,
    ) -> Result<(), EmulatorError> {
        self.inner.borrow_mut().brk_address = load_info.brk_address;

        core.add_intr_hook({
            let inner = self.inner.clone();
            move |uc, signal| {
                inner.borrow_mut().on_interrupt(uc, signal);
            }
        })?;
        Ok(())
    }
}

impl Inner {
    fn on_interrupt<'a, A: ArchT>(&mut self, core: &mut Engine<'a, A>, s: u32) {
        let arch = core.get_arch();
        let signal = intr_signal(arch);
        if signal != s {
            return;
        }

        let syscall_no = get_syscall(arch, core).unwrap();
        let call = syscall::SYSCALL
            .get(&(core.get_arch() as u8))
            .and_then(|v| v.get(&syscall_no));
        match call {
            None => {
                unimplemented!("no such syscall {} for {:?}", syscall_no, arch);
            }
            Some(call) => match SysCalls::from_str(call.as_str()) {
                Ok(c) => {
                    self.handle_syscall(core, c).unwrap();
                }
                Err(_e) => {
                    unimplemented!("syscall {} for {:?}", syscall_no, arch);
                }
            },
        }
    }

    fn handle_syscall<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        syscall: SysCalls,
    ) -> Result<(), EmulatorError> {
        assert_eq!(core.get_arch(), Arch::MIPS, "only support mips for now");
        let cc = core.get_data().env().cc();
        let retvalue = match syscall {
            SysCalls::SET_THREAD_AREA => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                self.set_thread_area(core, p0)?
            }
            SysCalls::SET_TID_ADDRESS => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                self.set_tid_address(core, p0)?
            }
            SysCalls::POLL => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                self.poll(core, p0, p1, p2)?
            }
            SysCalls::RT_SIGACTION => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                self.rt_sigaction(core, p0, p1, p2)?
            }
            SysCalls::RT_SIGPROCMASK => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                let p3 = cc.get_raw_param(core, 3, None)?;
                self.rt_sigprocmask(core, p0, p1, p2, p3)?
            }
            SysCalls::SIGALTSTACK => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                self.sigaltstack(core, p0, p1)?
            }
            SysCalls::SIGRETURN => self.sigreturn(core)?,
            SysCalls::RT_SIGRETURN => self.sigreturn(core)?,
            SysCalls::BRK => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                self.brk(core, p0)?
            }
            SysCalls::EXIT_GROUP => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                self.exit_group(core, p0)?
            }
            SysCalls::GETRANDOM => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                self.get_random(core, p0, p1)?
            }
            SysCalls::SCHED_GETAFFINITY => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;

                self.sched_getaffinity(core, p0, p1, p2)?
            }
            SysCalls::SCHED_YIELD => self.sched_yield(core)?,
            SysCalls::TKILL => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                self.tkill(core, p0, p1, p2)?
            }
            SysCalls::FUTEX => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                let p3 = cc.get_raw_param(core, 3, None)?;
                let p4 = cc.get_raw_param(core, 4, None)?;
                let p5 = cc.get_raw_param(core, 5, None)?;
                self.futex(core, p0, p1, p2, p3, p4, p5)?
            }
            SysCalls::EXIT => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                self.exit(core, p0)?
            }
            SysCalls::CLOCK_GETTIME => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 0, None)?;
                self.clock_gettime(core, p0, p1)?
            }
            SysCalls::MMAP2 => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                let p3 = cc.get_raw_param(core, 3, None)?;
                let p4 = cc.get_raw_param(core, 4, None)?;
                let p5 = cc.get_raw_param(core, 5, None)?;
                self.mmap2(core, p0, p1, p2, p3, p4, p5, 2)?
            }
            SysCalls::MREMAP => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                let p3 = cc.get_raw_param(core, 3, None)?;
                let p4 = cc.get_raw_param(core, 4, None)?;
                self.mremap(core, p0, p1, p2, p3, p4)?
            }
            SysCalls::MUNMAP => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                self.munmap(core, p0, p1)?
            }
            SysCalls::MPROTECT => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                self.mprotect(core, p0, p1, p2)?
            }
            SysCalls::MADVISE => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                self.madivse(core, p0, p1, p2)?
            }
            SysCalls::GETRLIMIT => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                self.getrlimit(core, p0, p1)?
            }
            SysCalls::SYSINFO => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                self.sysinfo(core, p0)?
            }
            SysCalls::SET_ROBUST_LIST => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                self.set_robust_list(core, p0, p1)?
            }
            SysCalls::PRLIMIT64 => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 1, None)?;
                let p3 = cc.get_raw_param(core, 1, None)?;
                self.prlimit64(core, p0, p1, p2, p3)?
            }
            SysCalls::OPEN => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                self.open(core, p0, p1, p2)?
            }
            SysCalls::READ => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                self.read(core, p0, p1, p2)?
            }
            SysCalls::WRITE => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                self.write(core, p0, p1, p2)?
            }
            SysCalls::CLOSE => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                self.close(core, p0)?
            }
            SysCalls::LSEEK => {
                let p0 = cc.get_raw_param(core, 0, None)?;
                let p1 = cc.get_raw_param(core, 1, None)?;
                let p2 = cc.get_raw_param(core, 2, None)?;
                self.lseek(core, p0, p1, p2)?
            }

            _ => {
                panic!("please handle syscall: {:?}", syscall);
            }
        };

        cc.set_return_value(core, retvalue as u64)?;

        Ok(())
    }
}

// pub trait LinuxSyscallHandler {
//     fn set_thread_area(&mut self, core: &mut impl Mach, u_info_addr: u64) -> Result<i64, uc_error>;
//     fn set_tid_address(core: &mut impl Mach, tidptr: u64) -> Result<i64, uc_error>;
//     fn poll<A: ArchT>(
//         core: &mut impl Mach,
//         fds: u64,
//         nfds: u64,
//         timeout: u64,
//     ) -> Result<i64, uc_error>;
//     fn rt_sigaction<A: ArchT>(
//         core: &mut impl Mach,
//         signum: u64,
//         act: u64,
//         oldact: u64,
//     ) -> Result<i64, uc_error>;
//     fn rt_sigprocmask<A: ArchT>(
//         core: &mut impl Mach,
//         how: u64,
//         nset: u64,
//         oset: u64,
//         sigsetsize: u64,
//     ) -> Result<i64, uc_error>;
//     fn syscall_signal(core: &mut impl Mach, sig: u64, sighandler: u64) -> Result<i64, uc_error>;
//     fn sigaltstack<A: ArchT>(core: &mut Core<A, Self>, ss: u64, oss: u64) -> Result<i64, uc_error>;
//     fn brk<A: ArchT>(core: &mut Core<A, Self>, inp: u64) -> Result<i64, uc_error>;
//     fn write<A: ArchT>(
//         core: &mut Core<A, Self>,
//         fd: u64,
//         buf: u64,
//         count: u64,
//     ) -> Result<i64, uc_error>;
//     fn exit_group<A: ArchT>(core: &mut Core<A, Self>, code: u64) -> Result<i64, uc_error>;
// }
//

impl Inner {
    fn set_thread_area<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
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

    fn set_tid_address<'a, A: ArchT>(
        &mut self,
        _core: &mut Engine<'a, A>,
        tidptr: u64,
    ) -> Result<i64, uc_error> {
        // TODO: implement thread management
        log::debug!("set_tid_address({})", tidptr);
        Ok(42)
    }

    fn poll<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
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

    fn rt_sigaction<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        signum: u64,
        act: u64,
        oldact: u64,
    ) -> Result<i64, uc_error> {
        if oldact != 0 {
            let arr = self
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
            self.sigaction_act.insert(signum, data);
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
    fn rt_sigprocmask<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
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

    // TODO: not implemented .
    fn sigaltstack<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        ss: u64,
        oss: u64,
    ) -> Result<i64, uc_error> {
        log::warn!(
            "not implemented, sigaltstack({}, {}) pc: {}",
            ss,
            oss,
            core.pc()?
        );
        Ok(0)
    }

    // TODO: not implemented .
    fn sigreturn<'a, A: ArchT>(&mut self, core: &mut Engine<'a, A>) -> Result<i64, uc_error> {
        log::warn!("not implemented, sigreturn pc: {}", core.pc()?);
        Ok(0)
    }

    fn brk<'a, A: ArchT>(&mut self, core: &mut Engine<'a, A>, inp: u64) -> Result<i64, uc_error> {
        log::debug!("brk({}) pc: {}", inp, core.pc()?);
        // current brk_address will be modified if inp is not NULL(zero)
        // otherwise, just return current brk_address
        if inp != 0 {
            let cur_brk_addr = self.brk_address;
            let new_brk_addr = align_up(inp, core.pagesize());
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
            self.brk_address = new_brk_addr;
        }
        Ok(self.brk_address as i64)
    }
    fn exit_group<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        code: u64,
    ) -> Result<i64, uc_error> {
        log::debug!("exit_group({}) pc: {}", code, core.pc()?);
        core.emu_stop()?;
        Ok(0)
    }
    fn get_random<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        buf: u64,
        buf_len: u64,
    ) -> Result<i64, uc_error> {
        let mut left = buf_len;
        while left > 0 {
            let mut n = RAND_SOURCE_LEN;
            if left < RAND_SOURCE_LEN {
                n = left;
            }
            log::debug!(
                "get_random({}) content: {:x?}",
                n,
                &RAND_SOURCE[0..n as usize]
            );
            Memory::write(core, buf, &RAND_SOURCE[0..n as usize])?;
            left -= n;
        }

        Ok(0)
    }
    fn sched_getaffinity<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        _pid: u64,
        _cpusetsize: u64,
        _mask: u64,
    ) -> Result<i64, uc_error> {
        log::warn!("not implemented, sched_getaffinity pc: {}", core.pc()?);
        Ok(0)
    }
    fn sched_yield<'a, A: ArchT>(&mut self, core: &mut Engine<'a, A>) -> Result<i64, uc_error> {
        log::warn!("not implemented, sched_yield pc: {}", core.pc()?);
        Ok(0)
    }
    fn tkill<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        _pid: u64,
        _cpusetsize: u64,
        _mask: u64,
    ) -> Result<i64, uc_error> {
        log::warn!("not implemented, tkill pc: {}", core.pc()?);
        Ok(0)
    }
    fn futex<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        _uaddr: u64,
        _op: u64,
        _val: u64,
        _timeout: u64,
        _uaddr2: u64,
        _val3: u64,
    ) -> Result<i64, uc_error> {
        log::warn!("not implemented, futex pc: {}", core.pc()?);
        Ok(0)
    }
    fn exit<'a, A: ArchT>(&mut self, core: &mut Engine<'a, A>, code: u64) -> Result<i64, uc_error> {
        log::debug!("exit: {}", code);
        core.emu_stop()?;
        Ok(0)
    }
    fn clock_gettime<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        clock_id: u64,
        tp: u64,
    ) -> Result<i64, uc_error> {
        log::debug!("clock_gettime: id {} tp: {}", clock_id, tp);
        // on 32 bits platform, 32bits for sec, 32bits for nsec.
        Memory::write(core, tp, vec![0u8; 8])?;
        Ok(0)
    }
    fn mmap2<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        addr: u64,
        length: u64,
        prot: u64,
        flags: u64,
        fd: u64,
        pgoffset: u64, // TODO no fd supports yet
        _ver: u8,
    ) -> Result<i64, uc_error> {
        log::debug!(
            "[mmap2] {}, {}, {}, {}, {}, {}",
            &addr,
            &length,
            &prot,
            &flags,
            &fd,
            &pgoffset
        );
        const MAP_FAILED: i64 = -1;

        const MAP_SHARED: u64 = 0x01;
        const MAP_FIXED: u64 = 0x10;
        const MAP_ANONYMOUS: u64 = 0x20;
        let arch = core.get_arch();
        // mask off perms bits that are not supported by unicorn
        let perms = Permission::from_bits_truncate(prot as u32);

        let page_size = core.pagesize();

        let mut mmap_base = align(addr as u32, page_size as u32) as u64;
        if flags & MAP_FIXED != 0 && mmap_base != addr {
            return Ok(MAP_FAILED);
        }

        let mmap_size =
            align_up((length - (addr & (page_size - 1))) as u32, page_size as u32) as u64;

        let mut need_map = true;
        if mmap_base != 0 {
            // already mapped.
            if Memory::is_mapped(core, mmap_base as u64, mmap_size as usize)? {
                // if map fixed, we just protect mem
                if flags & MAP_FIXED != 0 {
                    log::debug!("mmap2 - MAP_FIXED, mapping not needed");

                    Memory::mprotect(core, mmap_base as u64, mmap_size as usize, perms)?;
                    need_map = false;
                } else {
                    // or else, we need to reallocate mem somewhere else.
                    mmap_base = 0;
                }
            }
        }
        if need_map {
            if mmap_base == 0 {
                mmap_base = self.mmap_address;
                self.mmap_address = mmap_base + mmap_size;
            }

            log::debug!(
                "[mmap2] mapping for [{},{})",
                mmap_base,
                mmap_size + mmap_size
            );
            Memory::mem_map(
                core,
                MemRegion {
                    begin: mmap_base as u64,
                    end: (mmap_base + mmap_size) as u64,
                    perms,
                },
                Some("[syscall_mmap2]".to_string()),
            )?;

            // FIXME: MIPS32 Big Endian
            if arch == Arch::MIPS {
                Memory::write(core, mmap_base as u64, vec![0u8; mmap_size as usize])?;
            }
        }
        // TODO: should handle fd?
        log::warn!("[mmap2] fd {} not handled", fd);
        Ok(mmap_base as i64)
    }
    fn mremap<'a, A: ArchT>(
        &mut self,
        _core: &mut Engine<'a, A>,
        old_addr: u64,
        old_size: u64,
        new_size: u64,
        flags: u64,
        new_addr: u64,
    ) -> Result<i64, uc_error> {
        log::debug!(
            "[mremap] {} {} {} {} {}",
            old_addr,
            old_size,
            new_size,
            flags,
            new_addr
        );
        Ok(-1)
    }
    fn munmap<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        addr: u64,
        length: u64,
    ) -> Result<i64, uc_error> {
        log::debug!("[munmap] addr: {:#x}, length: {:#x}", addr, length);
        let length = align_up(length as u32, core.pagesize() as u32);
        Memory::mem_unmap(core, addr, length as usize)?;
        Ok(0)
    }
    fn mprotect<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        addr: u64,
        size: u64,
        prot: u64,
    ) -> Result<i64, uc_error> {
        let page_size = core.pagesize();
        let mmap_base = align(addr as u32, page_size as u32) as u64;
        let mmap_size = align_up((size - (addr & (page_size - 1))) as u32, page_size as u32) as u64;
        let perms = Permission::from_bits_truncate(prot as u32);

        log::debug!(
            "[mprotect] addr: {:#x}, size: {:#x}, perms: {:#x}",
            mmap_base,
            mmap_size,
            perms
        );
        Memory::mprotect(core, mmap_base, mmap_size as usize, perms)?;
        Ok(0)
    }

    fn madivse<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        _addr: u64,
        _length: u64,
        _advice: u64,
    ) -> Result<i64, uc_error> {
        log::warn!("not implemented, madvise pc: {}", core.pc()?);
        Ok(0)
    }

    fn getrlimit<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        res: u64,
        rlim: u64,
    ) -> Result<i64, uc_error> {
        log::debug!("[getrlimit] res: {:#x}, rlim: {:#x}", res, rlim);

        let rlimit = get_rlimit(res);
        Memory::write_ptr(
            core,
            rlim,
            (&rlimit as *const Rlimit) as u64,
            Some(core.pointer_size()),
        )?;
        Ok(0)
    }

    fn sysinfo<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        info: u64,
    ) -> Result<i64, uc_error> {
        log::debug!("[sysinfo] info: {:#x}", info);

        let i: SysInfo = Default::default();
        Memory::write_ptr(
            core,
            info,
            (&i as *const SysInfo) as u64,
            Some(core.pointer_size()),
        )?;
        Ok(0)
    }

    fn set_robust_list<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        _head_ptr: u64,
        _head_len: u64,
    ) -> Result<i64, uc_error> {
        log::warn!("not implemented, set_robust_list pc: {}", core.pc()?);
        Ok(0)
    }

    fn prlimit64<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        pid: u64,
        res: u64,
        new_limit: u64,
        old_limit: u64,
    ) -> Result<i64, uc_error> {
        log::debug!(
            "[prlimit64] pid: {:#x}, res: {:#x}, new_limit: {:#x}, old_limit: {:#x}",
            pid,
            res,
            new_limit,
            old_limit
        );
        if pid == 0 && new_limit == 0 {
            let rlimit = get_rlimit(res);
            Memory::write_ptr(
                core,
                old_limit,
                (&rlimit as *const Rlimit) as u64,
                Some(core.pointer_size()),
            )?;
            return Ok(0);
        }

        Ok(-1)
    }
    fn open<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        filename: u64,
        flags: u64,
        mode: u64,
    ) -> Result<i64, EmulatorError> {
        let path = read_string(core, filename, b"\x00")?;
        log::debug!("open({}, {}, {}) pc: {}", path, flags, mode, core.pc()?);
        let fd = return match open(path.as_str(), flags, mode) {
            Ok(fd) => Ok(fd),
            Err(e) => {
                log::warn!("failed to open ({}, {}, {}) pc: {:?}", path, flags, mode, e);
                Ok(-1)
            }
        };
    }
    fn write<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        fd: u64,
        buf: u64,
        count: u64,
    ) -> Result<i64, EmulatorError> {
        log::debug!("write({}, {}, {}) pc: {}", fd, buf, count, core.pc()?);
        let data = Memory::read(core, buf, count as usize)?;
        let size = return match write(fd, data.as_ptr() as usize as u64, count) {
            Ok(size) => Ok(size),
            Err(e) => {
                log::warn!("failed to write ({}, {}, {}) pc: {:?}", fd, buf, count, e);
                Ok(-1)
            }
        };
    }
    fn read<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        fd: u64,
        buf: u64,
        len: u64,
    ) -> Result<i64, EmulatorError> {
        log::debug!("read({}, {}, {}) pc: {}", fd, buf, len, core.pc()?);
        let host_buf = vec![0_u8; len as usize];
        let size = match read(fd, host_buf.as_ptr() as usize as u64, len) {
            Ok(size) => size,
            Err(e) => {
                log::warn!("failed to read ({}, {}, {}) pc: {:?}", fd, buf, len, e);
                return Ok(-1);
            }
        };
        Memory::write(core, buf, host_buf)?;
        Ok(size)
    }
    fn close<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        fd: u64,
    ) -> Result<i64, EmulatorError> {
        log::debug!("close({}) pc: {}", fd, core.pc()?);
        let ret = return match close(fd) {
            Err(e) => {
                log::warn!("failed to close ({}) pc: {:?}", fd, e);
                Ok(-1)
            }
            _ => Ok(0),
        };
    }
    fn lseek<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        fd: u64,
        offset: u64,
        whence: u64,
    ) -> Result<i64, EmulatorError> {
        log::debug!("lseek({}, {}, {}) pc: {}", fd, offset, whence, core.pc()?);
        let off = return match lseek(fd, offset, whence) {
            Ok(off) => Ok(off),
            Err(e) => {
                log::warn!("failed to lseek ({} {} {}) pc: {:?}", fd, offset, whence, e);
                Ok(-1)
            }
        };
    }
    fn fcntl<'a, A: ArchT>(
        &mut self,
        core: &mut Engine<'a, A>,
        fd: u64,
        cmd: u64,
        arg: u64,
    ) -> Result<i64, EmulatorError> {
        log::debug!("fcntl({}, {}, {}) pc: {}", fd, cmd, arg, core.pc()?);
        let off = return match fcntl(fd, cmd, arg) {
            Ok(off) => Ok(off),
            Err(e) => {
                log::warn!("failed to fcntl ({} {} {}) pc: {:?}", fd, cmd, arg, e);
                Ok(-1)
            }
        };
    }
}

fn get_rlimit(res: u64) -> Rlimit {
    let mut r0: u32 = u32::MAX;
    if res == 3 {
        // RLIMIT_STACK
        r0 = 196608 // 192KiB
    }
    let rlimit = Rlimit {
        cur: r0,
        max: u32::MAX,
    };
    return rlimit;
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
