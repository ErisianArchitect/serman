

use libc::{
    c_int,
    __errno_location,
};

// Errors:
//  * pipe:
//      * EMFILE
//        The per-process limit on the number of open file descriptors has been reached.
//      * ENFILE
//        The system-wide limit on the total number of open files has been reached.
//  * fcntl:
//      * EACCES or EAGAIN
//        Operation is prohibited by locks held by other processes.
//      * EAGAIN
//        The operation is prohibited because the file has been memory-mapped by another process.
//      * EBADF
//        fd is not an open file descriptor
//      * EINVAL
//        The value specified in op is not recognized by this kernel.
//  * poll:
//      * EAGAIN
//        The system failed to allocate kernel-internal resources. Program may wish to check for EAGAIN and
//        loop, just as with EINTR.
//      * EFAULT
//        The fds pints outside the process's accessible address space. The array given as argument was not
//        contained in the calling program's address space.
//      * EINTR
//        A signal occurred before any requested event. see (signal(7))
//      * EINVAL
//        The nfds value exceeds the RLIMIT_NOFILE value.
//      * ENOMEM
//        Unable to allocate memory for kernel data structures.
//  * read:
//      * EAGAIN
//        The file descriptor fd refers to a file other than a socket and has been marked nonblocking, and the
//        read would block.
//      * EWOULDBLOCK
//        The file descriptor fd refers to a socket and has been marked nonblocking, and the read would block.
//      * EBADF
//        fd is not a valid file descriptor or is not open for reading.
//      * EFAULT
//        buf is outside your accessible address space.
//      * EINTR
//        The call was interrupted by a signal before any data was read; see signal(7)
//      * EINVAL
//        fd is attached to an object which is unsuitable for reading; or the file was opened with the
//        O_DIRECT flag, and either the address speicfied in buf, the value specified in count, or the
//        file offset is not suitably aligned.
//      * EIO
//        I/O error. This will happen for example when the process is in a background process group, tries
//        to read from its controlling terminal, and either it is ignoring or blocking SIGTTIN or its process
//        group is orphaned. It may also occur when there is a low-level I/O error while reading from a disk or
//        tape. A further possible cause of EIO on networked filesystems is when an advisory lock had been
//        taken out on the file descriptor and this lock has been lost. See the Lost locks section of fcntl(2)
//        for further details.
//      * EISDIR
//        fd refers to directory.
//      * Other errors may occur depending on the object connected to fd.
//  * write:
//      * EWOULDBLOCK
//      * EBADF
//      * EDESTADDRREQ
//      * EDQUOT
//      * EFAULT
//      * EFBIG
//      * EINTR
//      * EINVAL
//      * EIO
//      * ENOSPC
//      * EPERM
//      * EPIPE
//      * Other errors may occur depending on the object connected to fd.
//  * fork:
//      * EAGAIN
//      * ENOMEM
//      * ENOSYS
//  * waitpid:
//      * EAGAIN
//      * ECHILD
//      * EINTR
//      * EINVAL
//      * ESRCH
#[must_use]
#[inline(always)]
pub fn errno() -> c_int {
    unsafe { *__errno_location() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
#[cfg_attr(
    any(target_arch = "avr", target_arch = "msp430"),
    repr(i16),
)]
#[cfg_attr(
    not(any(target_arch = "avr", target_arch = "msp430")),
    repr(i32),
)]
pub enum PipeError {
    #[error("Per-process limit reached on number of open file descriptor. (See EMFILE)")]
    ProcessFileLimit = libc::EMFILE,
    #[error("System-wide limit on the number of open descriptors has been reached. (See ENFILE)")]
    SystemFileLimit = libc::ENFILE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum Error {
    #[error("Pipe Error: {0}")]
    Pipe(#[from] PipeError),
    #[error("errno({0})")]
    Errno(c_int),
}

impl Error {
    pub const fn match_error(errno: c_int) -> Self {
        match errno {
            // pipe errors
            libc::EMFILE => Self::Pipe(PipeError::ProcessFileLimit),
            libc::ENFILE => Self::Pipe(PipeError::SystemFileLimit),
            
            // TODO: Rest of the errors.
            // Unrecognized error.
            errno => Self::Errno(errno),
        }
    }
    
    pub fn get_error() -> Self {
        Self::match_error(errno())
    }
    
    pub fn get_pipe_error() -> Self {
        match errno() {
            libc::EMFILE => Self::Pipe(PipeError::ProcessFileLimit),
            libc::ENFILE => Self::Pipe(PipeError::SystemFileLimit),
            // fallback on match_error for full error matching.
            errno => Self::match_error(errno),
        }
    }
}
