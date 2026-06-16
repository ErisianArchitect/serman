
use libc::{
    c_int,
};

#[cfg(not(unix))]
compile_error!("This library is for unix systems, and the target is not a unix system.");


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

/// libc errno wrapper. See libc error values for more information.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
#[error("Errno({0})")]
pub struct Errno(c_int);

impl Errno {
    #[must_use]
    #[inline(always)]
    pub const fn new(value: c_int) -> Self {
        Self(value)
    }
    
    #[must_use]
    #[inline(always)]
    pub fn get() -> Self {
        unsafe {
            let ptr = libc::__errno_location();
            Self(*ptr)
        }
    }

    #[must_use]
    #[inline(always)]
    pub fn value(self) -> c_int {
        self.0
    }
}

#[repr(C)]
struct Reader {
    fd: c_int,
    open: bool,
}

#[repr(C)]
struct Writer {
    fd: c_int,
    open: bool,
}
