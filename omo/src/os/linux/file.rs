use crate::{
    errors::{EmulatorError, from_raw_syscall_ret},
    os::linux::syscall::*,
};

pub fn open(path: *const u8, flags: u64, mode: u64) -> Result<i64, EmulatorError> {
    let flags = flags & 0xffffffff;
    let open_mode = mode & 0x7fffffff;
    let fd = unsafe { syscall_4(LinuxSysCalls::Open as u64, path as u64, flags, open_mode, 0) };
    if fd < 0 {
        Err(from_raw_syscall_ret(fd))
    } else {
        Ok(fd)
    }
}

pub fn read(fd: u64, buf: *mut u8, size: u64) -> Result<i64, EmulatorError> {
    let size = unsafe { syscall_4(LinuxSysCalls::Read as u64, fd, buf as u64, size, 0) };
    if size < 0 {
        Err(from_raw_syscall_ret(size))
    } else {
        Ok(size)
    }
}

pub fn write(fd: u64, data: *const u8, len: u64) -> Result<i64, EmulatorError> {
    let size = unsafe { syscall_4(LinuxSysCalls::Write as u64, fd, data as u64, len, 0) };
    if size < 0 {
        Err(from_raw_syscall_ret(size))
    } else {
        Ok(size)
    }
}

pub fn close(fd: u64) -> Result<i64, EmulatorError> {
    let ret = unsafe { syscall_4(LinuxSysCalls::Close as u64, fd, 0, 0, 0) };
    log::debug!("close {}, ret: {}", fd, ret);
    if ret < 0 {
        Err(from_raw_syscall_ret(ret))
    } else {
        Ok(ret)
    }
}

pub fn lseek(fd: u64, offset: u64, whence: u64) -> Result<i64, EmulatorError> {
    let off = unsafe { syscall_4(LinuxSysCalls::Lseek as u64, fd, offset, whence, 0) };
    if off < 0 {
        Err(from_raw_syscall_ret(off))
    } else {
        Ok(off)
    }
}

pub fn fcntl(fd: u64, cmd: u64, arg: u64) -> Result<i64, EmulatorError> {
    let ret = unsafe { syscall_4(LinuxSysCalls::Fcntl as u64, fd, cmd, arg, 0) };
    if ret < 0 {
        Err(from_raw_syscall_ret(ret))
    } else {
        Ok(ret)
    }
}

pub fn readlink(path: *const u8, buf: *mut u8, buf_size: u64) -> Result<i64, EmulatorError> {
    let ret = unsafe {
        syscall_4(
            LinuxSysCalls::Readlink as u64,
            path as u64,
            buf as u64,
            buf_size,
            0)
    };
    if ret < 0 {
        Err(from_raw_syscall_ret(ret))
    } else {
        Ok(ret)
    }
}

pub fn stat(path: *const u8, stat_buf: *mut StatX8664) -> Result<i64, EmulatorError> {
    let ret = unsafe { syscall_4(LinuxSysCalls::Stat as u64, path as u64, stat_buf as u64, 0, 0) };
    if ret < 0 {
        Err(from_raw_syscall_ret(ret))
    } else {
        Ok(ret)
    }
}

pub fn fstat(fd: u64, stat_buf: *mut StatX8664) -> Result<i64, EmulatorError> {
    let ret = unsafe { syscall_4(LinuxSysCalls::Fstat as u64, fd, stat_buf as u64, 0, 0) };
    if ret < 0 {
        Err(from_raw_syscall_ret(ret))
    } else {
        Ok(ret)
    }
}

pub fn lstat(path: *const u8, stat_buf: *mut StatX8664) -> Result<i64, EmulatorError> {
    let ret = unsafe { syscall_4(LinuxSysCalls::Lstat as u64, path as u64, stat_buf as u64, 0, 0) };
    if ret < 0 {
        Err(from_raw_syscall_ret(ret))
    } else {
        Ok(ret)
    }
}

pub fn fstatat64(
    dir_fd: u64,
    path: *const u8,
    stat_buf: *mut StatX8664,
    flags: u64,
) -> Result<i64, EmulatorError> {
    let ret = unsafe {
        syscall_4(
            LinuxSysCalls::Newfstatat as u64,
            dir_fd,
            path as u64,
            stat_buf as u64,
            flags,
        )
    };
    if ret < 0 {
        Err(from_raw_syscall_ret(ret))
    } else {
        Ok(ret)
    }
}

pub fn ioctl(fd: u64, cmd: u64, arg: u64) -> Result<i64, EmulatorError> {
    let ret = unsafe { syscall_4(LinuxSysCalls::Ioctl as u64, fd, cmd, arg, 0) };
    if ret < 0 {
        Err(from_raw_syscall_ret(ret))
    } else {
        Ok(ret)
    }
}
