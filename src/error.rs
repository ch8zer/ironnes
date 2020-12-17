use thiserror::Error;

pub type IronNesResult<T> = std::result::Result<T, IronNesError>;

#[derive(Error, Debug)]
pub enum IronNesError {
    #[error("Error reading cartridge contents")]
    CartridgeError,
    #[error("MemError: {0}")]
    MemoryError(String),
    #[error("Instruction is not supported")]
    IllegalInstruction,
    #[error(transparent)]
    Other(#[from] std::io::Error),
}
