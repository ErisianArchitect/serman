

use libc::{
    c_int,
    __errno_location,
};

// Errors:

#[must_use]
#[inline(always)]
pub fn errno() -> c_int {
    unsafe { *__errno_location() }
}


// SPDX-License-Identifier: Linux-man-pages-copyleft
//  * pipe:
//      * EMFILE
//        The per-process limit on the number of open file descriptors has been reached.
//      * ENFILE
//        The system-wide limit on the total number of open files has been reached.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum PipeError {
    #[error("Per-process limit reached on number of open file descriptor. (See EMFILE)")]
    ProcessFileLimit,
    #[error("System-wide limit on the number of open descriptors has been reached. (See ENFILE)")]
    SystemFileLimit,
    #[error("Other error: errno({0}, 0x{0:X})")]
    Other(c_int),
}

pub type PipeResult<T = (), E = PipeError> = std::result::Result<T, E>;

impl PipeError {
    #[must_use]
    #[inline]
    pub fn match_error(errno: c_int) -> Self {
        match errno {
            libc::EMFILE => PipeError::ProcessFileLimit,
            libc::ENFILE => PipeError::SystemFileLimit,
            other => PipeError::Other(other),
        }
    }
    
    #[must_use]
    #[inline]
    pub fn get_err<T>() -> Result<T, Self> {
        Err(Self::match_error(errno()))
    }
}

// SPDX-License-Identifier: Linux-man-pages-copyleft
//  * fcntl:
//      * EACCES or EAGAIN
//        Operation is prohibited by locks held by other processes.
//      * EAGAIN
//        The operation is prohibited because the file has been memory-mapped by another process.
//      * EBADF
//        fd is not an open file descriptor
//      * EINVAL
//        The value specified in op is not recognized by this kernel.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum FdError {
    #[error("Operation is prohibited by locks held by other processes. (See EACCES")]
    Prohibited,
    #[error("File descriptor is not an open file descriptor. (See EBADF)")]
    BadFile,
    #[error("Other error: errno({0}, 0x{0:X})")]
    Other(c_int),
}

pub type FdResult<T = (), E = FdError> = std::result::Result<T, E>;

impl FdError {
    #[must_use]
    #[inline]
    pub fn match_error(errno: c_int) -> Self {
        match errno {
            libc::EACCES => Self::Prohibited,
            libc::EBADF => Self::BadFile,
            other => Self::Other(other),
        }
    }

    #[must_use]
    #[inline]
    pub fn get_err<T>() -> Result<T, Self> {
        Err(Self::match_error(errno()))
    }
}

// SPDX-License-Identifier: Linux-man-pages-copyleft
//  * poll:
//      * EAGAIN
//        The system failed to allocate kernel-internal resources. Program may wish to check for EAGAIN and
//        loop, just as with EINTR.
//      * EINTR
//        A signal occurred before any requested event. see (signal(7))
//      * ENOMEM
//        Unable to allocate memory for kernel data structures.
// NOTE: For `poll` errors, you only need to handle the above three. For EAGAIN and EINTR, just try again.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum PollError {
    #[error("System failed to allocate kernel-internal resources. It might be advisable to try again. (See EAGAIN)")]
    TryAgain,
    #[error("A signal occurred in the calling process before any requested event. (See EINTR)")]
    Interrupt,
    #[error("Unable to allocate memory. (See ENOMEM)")]
    OutOfMemory,
    #[error("Other error: errno({0}, 0x{0:X})")]
    Other(c_int),
}

pub type PollResult<T = (), E = PollError> = std::result::Result<T, E>;

impl PollError {
    #[must_use]
    #[inline]
    pub fn match_error(errno: c_int) -> Self {
        match errno {
            libc::EAGAIN => Self::TryAgain,
            libc::EINTR => Self::Interrupt,
            libc::ENOMEM => Self::OutOfMemory,
            other => Self::Other(other),
        }
    }

    #[must_use]
    #[inline]
    pub fn get_err<T>() -> Result<T, Self> {
        Err(Self::match_error(errno()))
    }
}

// SPDX-License-Identifier: Linux-man-pages-copyleft
//  * read:
//      * EWOULDBLOCK
//        The file descriptor fd refers to a socket and has been marked nonblocking, and the read would block.
//      * EBADF
//        fd is not a valid file descriptor or is not open for reading.
//      * EINTR
//        The call was interrupted by a signal before any data was read; see signal(7)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum ReadError {
    #[error("The file descriptor is marked as non-blocking and the operation would block. Advised to try again soon. (See EWOULDBLOCK)")]
    WouldBlock,
    #[error("File descriptor is not a valid file descriptor or is not open for reading. (See EBADF)")]
    BadFile,
    #[error("The call was interrupted by a signal before any data was read. (See EINTR)")]
    Interrupt,
    #[error("I/O error. (See EIO)")]
    IoError,
    #[error("Other error: errno({0}, 0x{0:X})")]
    Other(c_int),
}

pub type ReadResult<T = (), E = ReadError> = std::result::Result<T, E>;

impl ReadError {
    #[must_use]
    #[inline]
    pub fn match_error(errno: c_int) -> Self {
        match errno {
            libc::EWOULDBLOCK => Self::WouldBlock,
            libc::EBADF => Self::BadFile,
            libc::EINTR => Self::Interrupt,
            libc::EIO => Self::IoError,
            other => Self::Other(other),
        }
    }

    #[must_use]
    #[inline]
    pub fn get_err<T>() -> Result<T, Self> {
        Err(Self::match_error(errno()))
    }
}

// SPDX-License-Identifier: Linux-man-pages-copyleft
//  * write:
//      * EWOULDBLOCK
//        The file descriptor fd refers to a socket and has been marked nonblocking, and the write would block.
//      * EBADF
//        fd is not a valid file descriptor or is not open for writing.
//      * EINTR
//        The call was interrupted by a signal before any data was written; see signal(7).
//      * EIO
//        A low-level I/O error occurred while modifying the inode.  This error may relate to the write-back  of  data  written  by  an  earlier
//        write(),  which  may  have been issued to a different file descriptor on the same file.  Since Linux 4.13, errors from write-back come
//        with a promise that they may be reported by subsequent.  write() requests, and will be reported by a subsequent fsync(2)  (whether  or
//        not  they  were also reported by write()).  An alternate cause of EIO on networked filesystems is when an advisory lock had been taken
//        out on the file descriptor and this lock has been lost.  See the Lost locks section of fcntl(2) for further details.
//      * ENOSPC
//        The device containing the file referred to by fd has no room for the data.
//      * EPIPE
//        fd is connected to a pipe or socket whose reading end is closed.  When this happens the writing process will also  receive  a  SIGPIPE
//        signal.  (Thus, the write return value is seen only if the program catches, blocks or ignores this signal.)
//      * Other errors may occur depending on the object connected to fd.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum WriteError {
    #[error("The file descriptor is marked as non-blocking and the operation would block. Advised to try again soon. (See EWOULDBLOCK)")]
    WouldBlock,
    #[error("File descriptor is not a valid file descriptor or is not open for writing. (See EBADF)")]
    BadFile,
    #[error("The call was interrupted by a signal before any data was written. (See EINTR)")]
    Interrupt,
    #[error("I/O error. (See EIO)")]
    IoError,
    #[error("Read end of the pipe was closed. (See EPIPE)")]
    ReadEndClosed,
    #[error("Other error: errno({0}, 0x{0:X})")]
    Other(c_int),
}

pub type WriteResult<T = (), E = WriteError> = std::result::Result<T, E>;

impl WriteError {
    #[must_use]
    #[inline]
    pub fn match_error(errno: c_int) -> Self {
        match errno {
            libc::EWOULDBLOCK => Self::WouldBlock,
            libc::EBADF => Self::BadFile,
            libc::EINTR => Self::Interrupt,
            libc::EIO => Self::IoError,
            libc::EPIPE => Self::ReadEndClosed,
            other => Self::Other(other),
        }
    }
    
    #[must_use]
    #[inline]
    pub fn get_err<T>() -> Result<T, Self> {
        Err(Self::match_error(errno()))
    }
}

// SPDX-License-Identifier: Linux-man-pages-copyleft
//  * fork:
//      * ENOMEM
//        fork() failed to allocate the necessary kernel structures because memory is tight.
//      * ENOSYS
//        fork() is not supported on this platform (for example, hardware without a Memory-Management Unit).

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum ForkError {
    #[error("Unable to allocate memory. (See ENOMEM)")]
    OutOfMemory,
    #[error("Fork is not supported on this platform. (See ENOSYS)")]
    ForkNotSupported,
    #[error("Other error: errno({0}, 0x{0:X})")]
    Other(c_int),
}

pub type ForkResult<T = (), E = ForkError> = std::result::Result<T, E>;

impl ForkError {
    #[must_use]
    #[inline]
    pub fn match_error(errno: c_int) -> Self {
        match errno {
            libc::ENOMEM => Self::OutOfMemory,
            libc::ENOSYS => Self::ForkNotSupported,
            other => Self::Other(other),
        }
    }
    
    #[must_use]
    #[inline]
    pub fn get_err<T>() -> Result<T, Self> {
        Err(Self::match_error(errno()))
    }
}

// SPDX-License-Identifier: Linux-man-pages-copyleft
//  * waitpid:
//      * ECHILD
//        (for  waitpid()  or waitid()) The process specified by pid (waitpid()) or idtype and id (waitid()) does not exist or is not a child of
//        the calling process.  (This can happen for one's own child if the action for SIGCHLD is set to SIG_IGN.  See also the Linux Notes sec‐
//        tion about threads.)
//      * EINTR
//        WNOHANG was not set and an unblocked signal or a SIGCHLD was caught; see signal(7).
//      * ESRCH
//        pid is equal to INT_MIN.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum WaitError {
    #[error("The file descriptor is marked as non-blocking and the operation would block. Advised to try again soon. (See EWOULDBLOCK)")]
    WouldBlock,
    #[error("WNOHANG was not set and an unblocked signal or a SIGCHLD was caught. (See EINTR)")]
    Interrupt,
    #[error("Other error: errno({0}, 0x{0:X})")]
    Other(c_int),
}

pub type WaitResult<T = (), E = WaitError> = std::result::Result<T, E>;

impl WaitError {
    #[must_use]
    #[inline]
    pub fn match_error(errno: c_int) -> Self {
        match errno {
            libc::EWOULDBLOCK => Self::WouldBlock,
            libc::EINTR => Self::Interrupt,
            other => Self::Other(other),
        }
    }
    
    #[must_use]
    #[inline]
    pub fn get_err<T>() -> Result<T, Self> {
        Err(Self::match_error(errno()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum Error {
    #[error("Pipe Error: {0}")]
    Pipe(#[from] PipeError),
    #[error("File Descriptor Error: {0}")]
    Fd(#[from] FdError),
    #[error("Poll Error: {0}")]
    Poll(#[from] PollError),
    #[error("Read Error: {0}")]
    Read(#[from] ReadError),
    #[error("Write Error: {0}")]
    Write(#[from] WriteError),
    #[error("Fork Error: {0}")]
    Fork(#[from] ForkError),
    #[error("Wait Error: {0}")]
    Wait(#[from] WaitError),
    #[error("errno({0})")]
    Errno(c_int),
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;
