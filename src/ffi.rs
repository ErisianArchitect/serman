
use libc::{
    // Types
    c_int, c_uint,
    pollfd,
    // Functions
    // pipe,
    // fcntl,
    // poll,
    // read,
    // write,
    // fork,
    // waitpid,
    __errno_location,
};

use crate::error::{
    errno,
    Error,
    Result,
    FdError,
    FdResult,
    ForkError,
    ForkResult,
    PipeError,
    PipeResult,
    PollError,
    PollResult,
    ReadError,
    ReadResult,
    WaitError,
    WaitResult,
    WriteError,
    WriteResult,
};

#[repr(C, align(8))]
struct PipeFds {
    pub reader: c_int,
    pub writer: c_int,
}

impl PipeFds {
    #[must_use]
    #[inline(always)]
    pub const fn zeroed() -> Self {
        Self {
            reader: 0,
            writer: 0,
        }
    }

    #[must_use]
    #[inline(always)]
    fn as_mut_fds(&mut self) -> *mut c_int {
        self as *mut Self as _
    }
}

#[inline]
pub fn pipe() -> PipeResult<PipeFds> {
    let mut fds = PipeFds::zeroed();
    let pipe_result = unsafe { libc::pipe(fds.as_mut_fds()) };
    if pipe_result == -1 {
        return PipeError::get_err();
    }
    Ok(fds)
}
