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
    WRITE,
    GETPID,
    _LLSEEK,
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
}

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
