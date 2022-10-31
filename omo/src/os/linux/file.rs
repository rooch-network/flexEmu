use std::io;

use crate::{errors::EmulatorError, os::linux::syscall::*};

pub fn open(path: &str, flags: u64, mode: u64) -> Result<i64, EmulatorError> {
    let open_mode = mode & 0x7fffffff;
    let fd = syscall_3(
        LinuxSysCalls::Open as u64,
        path.as_ptr() as u64,
        flags,
        open_mode,
    );
    if fd == -1 {
        Err(EmulatorError::IOError(io::Error::last_os_error()))
    } else {
        Ok(fd)
    }
}

pub fn read(fd: u64, buf: u64, size: u64) -> Result<i64, EmulatorError> {
    let size = syscall_3(LinuxSysCalls::Read as u64, fd, buf, size);
    if size == -1 {
        Err(EmulatorError::IOError(io::Error::last_os_error()))
    } else {
        Ok(size)
    }
}

pub fn write(fd: u64, data: u64, len: u64) -> Result<i64, EmulatorError> {
    let size = syscall_3(LinuxSysCalls::Write as u64, fd, data, len);
    if size == -1 {
        Err(EmulatorError::IOError(io::Error::last_os_error()))
    } else {
        Ok(size)
    }
}

pub fn close(fd: u64) -> Result<i64, EmulatorError> {
    let ret = syscall_1(LinuxSysCalls::Close as u64, fd);
    if ret == -1 {
        Err(EmulatorError::IOError(io::Error::last_os_error()))
    } else {
        Ok(ret)
    }
}

pub fn lseek(fd: u64, offset: u64, whence: u64) -> Result<i64, EmulatorError> {
    let off = syscall_3(LinuxSysCalls::Lseek as u64, fd, offset, whence);
    if off == -1 {
        Err(EmulatorError::IOError(io::Error::last_os_error()))
    } else {
        Ok(off)
    }
}

pub fn fcntl(fd: u64, cmd: u64, arg: u64) -> Result<i64, EmulatorError> {
    let ret = syscall_3(LinuxSysCalls::Fcntl as u64, fd, cmd, arg);
    if ret == -1 {
        Err(EmulatorError::IOError(io::Error::last_os_error()))
    } else {
        Ok(ret)
    }
}

pub fn readlink(path: &str, buf: u64, buf_size: u64) -> Result<i64, EmulatorError> {
    let ret = syscall_3(
        LinuxSysCalls::Readlink as u64,
        path.as_ptr() as u64,
        buf,
        buf_size,
    );
    if ret == -1 {
        Err(EmulatorError::IOError(io::Error::last_os_error()))
    } else {
        Ok(ret)
    }
}

pub fn stat(path: &str, stat_buf: u64) -> Result<i64, EmulatorError> {
    let ret = syscall_2(LinuxSysCalls::Stat as u64, path.as_ptr() as u64, stat_buf);
    if ret == -1 {
        Err(EmulatorError::IOError(io::Error::last_os_error()))
    } else {
        Ok(ret)
    }
}

pub fn fstat(fd: u64, stat_buf: u64) -> Result<i64, EmulatorError> {
    let ret = syscall_2(LinuxSysCalls::Fstat as u64, fd, stat_buf);
    if ret == -1 {
        Err(EmulatorError::IOError(io::Error::last_os_error()))
    } else {
        Ok(ret)
    }
}

pub fn lstat(path: &str, stat_buf: u64) -> Result<i64, EmulatorError> {
    let ret = syscall_2(LinuxSysCalls::Lstat as u64, path.as_ptr() as u64, stat_buf);
    if ret == -1 {
        Err(EmulatorError::IOError(io::Error::last_os_error()))
    } else {
        Ok(ret)
    }
}

pub fn fstatat64(dir_fd: u64, path: &str, stat_buf: u64, flags: u64) -> Result<i64, EmulatorError> {
    let ret = syscall_4(
        LinuxSysCalls::Newfstatat as u64,
        dir_fd,
        path.as_ptr() as u64,
        stat_buf,
        flags,
    );
    if ret == -1 {
        Err(EmulatorError::IOError(io::Error::last_os_error()))
    } else {
        Ok(ret)
    }
}

pub fn ioctl(fd: u64, cmd: u64, arg: u64) -> Result<i64, EmulatorError> {
    let ret = syscall_3(LinuxSysCalls::Ioctl as u64, fd, cmd, arg);
    if ret == -1 {
        Err(EmulatorError::IOError(io::Error::last_os_error()))
    } else {
        Ok(ret)
    }
}
