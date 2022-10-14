use std::io;

use crate::{errors::EmulatorError, syscall::*};

pub struct OFile {
    path: String,
    fd: i64,
}

impl OFile {
    pub fn open(self, path: &str, flags: u64, mode: u64) -> Result<Self, EmulatorError> {
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
            Ok(OFile {
                path: path.to_string(),
                fd,
            })
        }
    }
}
