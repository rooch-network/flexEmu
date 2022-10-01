use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use strum::{EnumString, EnumVariantNames};
use unicorn_engine::unicorn_const::Arch;

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

    WRITE,
    _LLSEEK,
}

pub struct Rlimit {
    pub cur: u32,
    pub max: u32,
}

#[repr(C)]
pub struct SysInfo {
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

impl Default for SysInfo {
    fn default() -> SysInfo {
        SysInfo {
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

// struct sysinfo {
//     __kernel_long_t uptime;		/* Seconds since boot */
//     __kernel_ulong_t loads[3];	/* 1, 5, and 15 minute load averages */
//     __kernel_ulong_t totalram;	/* Total usable main memory size */
//     __kernel_ulong_t freeram;	/* Available memory size */
//     __kernel_ulong_t sharedram;	/* Amount of shared memory */
//     __kernel_ulong_t bufferram;	/* Memory used by buffers */
//     __kernel_ulong_t totalswap;	/* Total swap space size */
//     __kernel_ulong_t freeswap;	/* swap space still available */
//     __u16 procs;		   	/* Number of current processes */
//     __u16 pad;		   	/* Explicit padding for m68k */
//     __kernel_ulong_t totalhigh;	/* Total high memory size */
//     __kernel_ulong_t freehigh;	/* Available high memory size */
//     __u32 mem_unit;			/* Memory unit size in bytes */
//     char _f[20-2*sizeof(__kernel_ulong_t)-sizeof(__u32)];	/* Padding: libc5 uses this.. */
// };

impl SysCalls {
    pub fn param_num(&self) -> usize {
        match self {
            Self::WRITE => 3,
            SysCalls::SET_THREAD_AREA => 1,
            SysCalls::SET_TID_ADDRESS => 1,
            SysCalls::POLL => 3,
            SysCalls::RT_SIGACTION => 3,
            SysCalls::RT_SIGPROCMASK => 4,
            SysCalls::SIGALTSTACK => 2,
            SysCalls::SIGRETURN => 0,
            SysCalls::RT_SIGRETURN => 0,
            SysCalls::BRK => 1,
            SysCalls::EXIT_GROUP => 1,
            SysCalls::GETRANDOM => 1,
            SysCalls::FUTEX => 6,
            SysCalls::SCHED_GETAFFINITY => 3,
            SysCalls::SCHED_YIELD => 0,
            SysCalls::TKILL => 3,
            _ => todo!(),
        }
    }
}
