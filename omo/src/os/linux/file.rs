use std::{
    fs::{File, OpenOptions},
    io::{stderr, stdin, stdout, Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd, RawFd},
        unix::fs::OpenOptionsExt,
    },
};

use log::{debug, info};

use crate::os::linux::{
    file::FileFlags::{O_ACCMODE, O_RDONLY, O_RDWR, O_WRONLY},
    syscall::*,
};

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn open(path: &mut String, flags: u64, mode: u64) -> i64 {
    let flags = flags & 0xffffffff;
    let open_mode = mode & 0x7fffffff;

    let mut c_path = path.as_bytes().to_vec();
    c_path.extend_from_slice(b"\x00");

    unsafe { syscall_4(LinuxSysCalls::Open as u64, path as u64, flags, open_mode, 0) }
}

#[repr(u64)]
pub enum FileFlags {
    O_ACCMODE = 0x3,
    O_RDONLY = 0x0,
    O_WRONLY = 0x1,
    O_RDWR = 0x2,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn open(path: &mut String, flags: u64, mode: u64) -> i64 {
    let flags = flags & 0xffffffff;
    let open_mode = mode & 0x7fffffff;

    let file = OpenOptions::new()
        .custom_flags(flags as i32)
        .read(
            (flags & O_ACCMODE as u64 == O_RDONLY as u64)
                || (flags & O_ACCMODE as u64 == O_RDWR as u64),
        )
        .write(
            (flags & O_ACCMODE as u64 == O_WRONLY as u64)
                || (flags & O_ACCMODE as u64 == O_RDWR as u64),
        )
        .mode(open_mode as u32)
        .open(path)
        .unwrap();

    file.as_raw_fd() as i64
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn read(fd: u64, buf: &mut Vec<u8>, size: u64) -> i64 {
    let buf = buf.as_ptr();
    unsafe { syscall_4(LinuxSysCalls::Read as u64, fd, buf as u64, size, 0) }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn read(fd: u64, buf: &mut Vec<u8>, _size: u64) -> i64 {
    if fd == 0 {
        return stdin().read(buf).unwrap() as i64;
    }

    let mut f = unsafe { File::from_raw_fd(fd as RawFd) };
    f.read(buf).unwrap() as i64
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn write(fd: u64, mut data: Vec<u8>, len: u64) -> i64 {
    let data = data.as_ptr();
    unsafe { syscall_4(LinuxSysCalls::Write as u64, fd, data as u64, len, 0) }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn write(fd: u64, mut data: Vec<u8>, len: u64) -> i64 {
    if fd == 1 {
        return stdout().write(&mut *data).unwrap() as i64;
    }
    if fd == 2 {
        return stderr().write(&mut *data).unwrap() as i64;
    }

    let mut f = unsafe { File::from_raw_fd(fd as RawFd) };
    f.write(&mut *data).unwrap() as i64
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn close(fd: u64) -> i64 {
    unsafe { syscall_4(LinuxSysCalls::Close as u64, fd, 0, 0, 0) }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn close(fd: u64) -> i64 {
    let mut f = unsafe { File::from_raw_fd(fd as RawFd) };
    drop(f);
    0
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn lseek(fd: u64, offset: u64, whence: u64) -> i64 {
    unsafe { syscall_4(LinuxSysCalls::Lseek as u64, fd, offset, whence, 0) }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn lseek(fd: u64, offset: u64, whence: u64) -> i64 {
    0 // TODO
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn fcntl(fd: u64, cmd: u64, arg: u64) -> i64 {
    unsafe { syscall_4(LinuxSysCalls::Fcntl as u64, fd, cmd, arg, 0) }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn fcntl(fd: u64, cmd: u64, arg: u64) -> i64 {
    0 // TODO
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
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

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn readlink(path: *const u8, buf: *mut u8, buf_size: u64) -> i64 {
    0 // TODO
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
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

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn stat(path: *const u8, stat_buf: *mut StatX8664) -> i64 {
    0 // TODO
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn fstat(fd: u64, stat_buf: *mut StatX8664) -> i64 {
    unsafe { syscall_4(LinuxSysCalls::Fstat as u64, fd, stat_buf as u64, 0, 0) }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn fstat(fd: u64, stat_buf: *mut StatX8664) -> i64 {
    0 // TODO
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
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

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn lstat(path: *const u8, stat_buf: *mut StatX8664) -> i64 {
    0 // TODO
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
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

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn fstatat64(dir_fd: u64, path: *const u8, stat_buf: *mut StatX8664, flags: u64) -> i64 {
    0 // TODO
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn ioctl(fd: u64, cmd: u64, arg: u64) -> i64 {
    unsafe { syscall_4(LinuxSysCalls::Ioctl as u64, fd, cmd, arg, 0) }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn ioctl(fd: u64, cmd: u64, arg: u64) -> i64 {
    0 // TODO
}
