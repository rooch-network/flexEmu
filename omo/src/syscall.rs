use std::arch::asm;

#[repr(u64)]
pub enum LinuxSysCalls {
    Read = 0,
    Write = 1,
    Open = 2,
    WriteV = 20,
}

pub fn syscall_3(num: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    let res;
    unsafe {
        asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rax") res,
        );
    }
    res
}
