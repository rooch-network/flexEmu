use std::{
    arch::asm,
    collections::{BTreeMap, HashMap},
};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use strum::{EnumString, EnumVariantNames};
use unicorn_engine::unicorn_const::Arch;

use crate::errors::from_raw_syscall_ret;

// x86_64 syscall trap.
#[repr(u64)]
pub enum LinuxSysCalls {
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,
    Stat = 4,
    Fstat = 5,
    Lstat = 6,
    Lseek = 8,
    Ioctl = 16,
    Fcntl = 72,
    Readlink = 89,
    Newfstatat = 262,
}

pub unsafe fn syscall_4(trap: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> i64 {
    let res;
    asm!(
    "syscall",
    in("rax") trap,
    in("rdi") arg1,
    in("rsi") arg2,
    in("rdx") arg3,
    in("r10") arg4,
    lateout("rax") res,
    );

    res
}

pub struct Rlimit {
    pub cur: u32,
    pub max: u32,
}

#[repr(C)]
pub struct StatMIPS {
    pub st_dev: u32,
    st_pad1: [i32; 3],
    pub st_ino: u32,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u32,
    st_pad2: [u32; 2],
    pub st_size: u32,
    st_pad3: u32,
    pub st_atime: u32,
    pub st_atime_ns: u32,
    pub st_mtime: u32,
    pub st_mtime_ns: u32,
    pub st_ctime: u32,
    pub st_ctime_ns: u32,
    pub st_blksize: u32,
    pub st_blocks: u32,
    st_pad4: [u32; 14],
}

impl Default for StatMIPS {
    fn default() -> StatMIPS {
        StatMIPS {
            st_dev: 0,
            st_pad1: [0; 3],
            st_ino: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_pad2: [0; 2],
            st_size: 0,
            st_pad3: 0,
            st_atime: 0,
            st_atime_ns: 0,
            st_mtime: 0,
            st_mtime_ns: 0,
            st_ctime: 0,
            st_ctime_ns: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_pad4: [0; 14],
        }
    }
}

#[repr(C)]
pub struct Stat64MIPS {
    pub st_dev: u32,
    st_pad0: [i32; 3],
    pub st_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u32,
    st_pad1: [u32; 3],
    pub st_size: u64,
    pub st_atime: i32,
    pub st_atime_ns: u32,
    pub st_mtime: u32,
    pub st_mtime_ns: u32,
    pub st_ctime: u32,
    pub st_ctime_ns: u32,
    pub st_blksize: u32,
    st_pad2: u32,
    pub st_blocks: i64,
}

impl Default for Stat64MIPS {
    fn default() -> Stat64MIPS {
        Stat64MIPS {
            st_dev: 0,
            st_pad0: [0; 3],
            st_ino: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_pad1: [0; 3],
            st_size: 0,
            st_pad2: 0,
            st_atime: 0,
            st_atime_ns: 0,
            st_mtime: 0,
            st_mtime_ns: 0,
            st_ctime: 0,
            st_ctime_ns: 0,
            st_blksize: 0,
            st_blocks: 0,
        }
    }
}

#[repr(C)]
pub struct StatX8664 {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_nlink: u64,
    pub st_mode: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    __pad0: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atime: u64,
    pub st_atime_ns: u64,
    pub st_mtime: u64,
    pub st_mtime_ns: u64,
    pub st_ctime: u64,
    pub st_ctime_ns: u64,
    __unused: [i64; 3],
}

#[repr(C)]
pub struct SysInfoMIPS {
    pub uptime: i32,
    pub loads: [u32; 3],
    pub total_ram: u32,
    pub free_ram: u32,
    pub shared_ram: u32,
    pub buffer_ram: u32,
    pub total_swap: u32,
    pub free_swap: u32,
    pub procs: u16,
    _padding0: u16,
    pub total_high: u32,
    pub free_high: u32,
    pub mem_unit: u32,
    _padding1: u64,
}

impl Default for SysInfoMIPS {
    fn default() -> SysInfoMIPS {
        SysInfoMIPS {
            uptime: 1234,
            loads: [2000, 2000, 2000],
            total_ram: 10000000,
            free_ram: 10000000,
            shared_ram: 10000000,
            buffer_ram: 0,
            total_swap: 0,
            free_swap: 0,
            procs: 1,
            _padding0: 0,
            total_high: 0,
            free_high: 0,
            mem_unit: 0,
            _padding1: 0,
        }
    }
}

const LINUX_SYSCALL_TABLE: &str = include_str!("linux_syscall_table.json");

fn parse_syscall_table(data: &str) -> BTreeMap<u8, BTreeMap<u64, String>> {
    let data: HashMap<String, BTreeMap<u64, String>> = serde_json::from_str(data).unwrap();
    let mut result: BTreeMap<_, _> = Default::default();
    for (k, v) in data {
        let arch = match k.to_lowercase().as_str() {
            "mips" => Arch::MIPS,
            _ => todo!(),
        };
        result.insert(arch as u8, v);
    }
    result
}

lazy_static! {
    pub static ref SYSCALL: BTreeMap<u8, BTreeMap<u64, String>> =
        parse_syscall_table(LINUX_SYSCALL_TABLE);
}

#[allow(non_camel_case_types)]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, EnumVariantNames, EnumString,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SysCalls {
    GETPID,
    SET_THREAD_AREA,
    SET_TID_ADDRESS,
    POLL,
    RT_SIGACTION,
    RT_SIGPROCMASK,
    SYSCALL_SIGNAL,
    SCHED_GETAFFINITY,
    SIGALTSTACK,
    SIGRETURN,
    RT_SIGRETURN,
    BRK,
    EXIT_GROUP,
    GETRANDOM,
    FUTEX,
    SCHED_YIELD,
    TKILL,
    GETTIMEOFDAY,
    CLOCK_GETTIME,
    EXIT,
    MUNMAP,
    MPROTECT,
    MREMAP,
    MMAP2,
    MADVISE,
    GETRLIMIT,
    SYSINFO,
    SET_ROBUST_LIST,
    PRLIMIT64,
    OPEN,
    READ,
    WRITE,
    CLOSE,
    LSEEK,
    FCNTL,
    READLINK,
    STAT,
    _LLSEEK,
    STAT64,
    FSTAT,
    FSTAT64,
    FCNTL64,
    LSTAT64,
    FSTATAT64,
    GETCWD,
    IOCTL,
    WRITEV,
}
