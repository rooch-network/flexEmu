use thiserror::Error;
use unicorn_engine::unicorn_const::uc_error;
#[derive(Error, Debug)]
pub enum EmulatorError {
    #[error("unicorn error {0:?}")]
    UcError(uc_error),
    #[error("loader error {0}")]
    LoaderError(#[from] goblin::error::Error),
    #[error("custom error {0}")]
    Custom(#[from] anyhow::Error),
}

impl From<uc_error> for EmulatorError {
    fn from(e: uc_error) -> Self {
        Self::UcError(e)
    }
}

pub type Result<T> = std::result::Result<T, EmulatorError>;
