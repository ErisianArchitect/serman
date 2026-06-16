
use libc::{
    c_int,
};

#[cfg(not(unix))]
compile_error!("This library is for unix systems, and the target is not a unix system.");

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

    #[inline(always)]
    fn err<T>(self) -> Result<T> {
        Err(Error::Errno(self))
    }

    #[inline(always)]
    fn get_err<T>() -> Result<T> {
        Errno::get().err()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FileDescriptorError {
    #[error("File descriptor is already in use.")]
    AlreadyUsed,
    #[error("File Descriptor is either invalid or closed.")]
    InvalidOrClosed,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("libc Error: {0}")]
    Errno(#[from] Errno),
    #[error("File Descriptor Error: {0}")]
    FileDescriptorError(#[from] FileDescriptorError),
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

struct FileDescriptor {
    fd: c_int,
}

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        match self.close() {
            Ok(()) => {},
            Err(err) => eprintln!("FileDescriptor Drop failed: {err}"),
        }
    }
}

impl FileDescriptor {
    /// Creates a new [FileDescriptor] from its raw value.
    /// 
    /// File descriptor must be both open and valid.
    #[must_use]
    #[inline(always)]
    pub fn new(fd: c_int) -> Result<Self> {
        let result = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if result == -1 {
            return Errno::get_err();
        }
        Ok(Self {
            fd,
        })
    }
    
    pub fn close(&mut self) -> Result {
        let result = unsafe { libc::fcntl(self.fd, libc::F_GETFD) };
        if result == -1 {
            return Ok(());
        }
        let close_result = unsafe { libc::close(self.fd) };
        if close_result < 0 {
            return Errno::get_err();
        }
        Ok(())
    }

    pub fn dup(&self) -> Result<Self> {
        let result = unsafe { libc::dup(self.fd) };
        if result < 0 {
            return Errno::get_err();
        }
        Ok(Self { fd: result })
    }

    pub fn dup2(&self, fd: c_int) -> Result<Self> {
        let result = unsafe { libc::fcntl(self.fd, libc::F_GETFD) };
        if result == -1 {
            return Err(Error::FileDescriptorError(FileDescriptorError::InvalidOrClosed));
        }
        let result = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if result != -1 {
            return Err(Error::FileDescriptorError(FileDescriptorError::AlreadyUsed));
        }
        let result = unsafe { libc::dup2(self.fd, fd) };
        if result < 0 {
            return Errno::get_err();
        }
        Ok(Self { fd })
    }

    pub fn rebind(&mut self, fd: c_int) -> Result {
        let new = self.dup2(fd)?;
        *self = new;
        Ok(())
    }
}

#[repr(C)]
pub struct Reader {
    fd: FileDescriptor,
}

#[repr(C)]
pub struct Writer {
    fd: FileDescriptor,
}

impl Reader {
    
}

pub fn pipe(read_fd: Option<c_int>, write_fd: Option<c_int>) -> Result<(Reader, Writer)> {
    let mut fds: [c_int; 2] = [0; 2];
    let result = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if result < 0 {
        return Errno::get_err();
    }
    let mut reader_fd = FileDescriptor { fd: fds[0] };
    let mut writer_fd = FileDescriptor { fd: fds[1] };

    if let Some(read_fd) = read_fd {
        reader_fd.rebind(read_fd)?;
    }

    if let Some(write_fd) = write_fd {
        writer_fd.rebind(write_fd)?;
    }
    
    Ok((
        Reader { fd: reader_fd },
        Writer { fd: writer_fd },
    ))
}
