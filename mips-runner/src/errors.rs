use unicorn_engine::unicorn_const::uc_error;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum EmulatorError {
    #[error("unicorn error {0:?}")]
    UcError(#[from] uc_error),
    #[error("loader error {0}")]
    LoaderError(#[from] goblin::error::Error),
}