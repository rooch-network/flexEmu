use crate::os::linux::syscall::*;

pub fn open(path: *const u8, flags: u64, mode: u64) -> i64 {
    let flags = flags & 0xffffffff;
    let open_mode = mode & 0x7fffffff;
    unsafe { syscall_4(LinuxSysCalls::Open as u64, path as u64, flags, open_mode, 0) }
}

pub fn read(fd: u64, buf: *mut u8, size: u64) -> i64 {
    unsafe { syscall_4(LinuxSysCalls::Read as u64, fd, buf as u64, size, 0) }
}

pub fn write(fd: u64, data: *const u8, len: u64) -> i64 {
    unsafe { syscall_4(LinuxSysCalls::Write as u64, fd, data as u64, len, 0) }
}

pub fn close(fd: u64) -> i64 {
    unsafe { syscall_4(LinuxSysCalls::Close as u64, fd, 0, 0, 0) }
}

pub fn lseek(fd: u64, offset: u64, whence: u64) -> i64 {
    unsafe { syscall_4(LinuxSysCalls::Lseek as u64, fd, offset, whence, 0) }
}

pub fn fcntl(fd: u64, cmd: u64, arg: u64) -> i64 {
    unsafe { syscall_4(LinuxSysCalls::Fcntl as u64, fd, cmd, arg, 0) }
}

pub fn readlink(path: *const u8, buf: *mut u8, buf_size: u64) -> i64 {
    unsafe {
        syscall_4(
            LinuxSysCalls::Readlink as u64,
            path as u64,
            buf as u64,
            buf_size,
            0,
        )
    }
}

pub fn stat(path: *const u8, stat_buf: *mut StatX8664) -> i64 {
    unsafe {
        syscall_4(
            LinuxSysCalls::Stat as u64,
            path as u64,
            stat_buf as u64,
            0,
            0,
        )
    }
}

pub fn fstat(fd: u64, stat_buf: *mut StatX8664) -> i64 {
    unsafe { syscall_4(LinuxSysCalls::Fstat as u64, fd, stat_buf as u64, 0, 0) }
}

pub fn lstat(path: *const u8, stat_buf: *mut StatX8664) -> i64 {
    unsafe {
        syscall_4(
            LinuxSysCalls::Lstat as u64,
            path as u64,
            stat_buf as u64,
            0,
            0,
        )
    }
}

pub fn fstatat64(dir_fd: u64, path: *const u8, stat_buf: *mut StatX8664, flags: u64) -> i64 {
    unsafe {
        syscall_4(
            LinuxSysCalls::Newfstatat as u64,
            dir_fd,
            path as u64,
            stat_buf as u64,
            flags,
        )
    }
}

pub fn ioctl(fd: u64, cmd: u64, arg: u64) -> i64 {
    unsafe { syscall_4(LinuxSysCalls::Ioctl as u64, fd, cmd, arg, 0) }
}
