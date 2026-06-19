
use libc::{
    // Types
    c_int, c_uint,
    pollfd,
    // Functions
    pipe,
    fcntl,
    poll,
    read,
    write,
    fork,
    waitpid,
    __errno_location,
};

use crate::error::errno;
