use std::{io, io::Error};

use thiserror::Error;
use unicorn_engine::unicorn_const::uc_error;

#[derive(Error, Debug)]
pub enum EmulatorError {
    #[error("unicorn error {0:?}")]
    UcError(uc_error),
    #[error("loader error {0}")]
    LoaderError(#[from] goblin::error::Error),
    #[error("io error {0}")]
    IOError(#[from] io::Error),
    #[error("custom error {0}")]
    Custom(#[from] anyhow::Error),
}

pub fn from_raw_syscall_ret(ret: i64) -> EmulatorError {
    EmulatorError::IOError(Error::from_raw_os_error(-ret as i32))   // raw ret is negative.
}

impl From<uc_error> for EmulatorError {
    fn from(e: uc_error) -> Self {
        Self::UcError(e)
    }
}

pub type Result<T> = std::result::Result<T, EmulatorError>;
