
#[cfg(not(unix))]
compile_error!("This library is for unix systems, and the target is not a unix system.");


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;
