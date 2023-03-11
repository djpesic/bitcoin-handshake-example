use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid config format")]
    ConfigFormatEror(#[from] figment::Error),
    #[error("Invalid config data: `{0}`")]
    ConfigDataEror(String),
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    SetLoggerError(#[from] log::SetLoggerError),
}
