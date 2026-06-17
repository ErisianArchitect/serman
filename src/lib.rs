
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
    pub fn filter_result(result: c_int) -> Result<c_int> {
        if result == -1 {
            return Self::get_err();
        }
        Ok(result)
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
    #[error("File Descriptor is not a reader.")]
    NotAReader,
    #[error("File Descriptor is not a writer.")]
    NotAWriter,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("libc Error: {0}")]
    Errno(#[from] Errno),
    #[error("File Descriptor Error: {0}")]
    FileDescriptor(#[from] FileDescriptorError),
    #[error("Both the read and write file descriptors are the same number.")]
    SameFileDescriptor,
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

impl FileDescriptorError {
    #[must_use]
    #[inline(always)]
    pub const fn err<T>(self) -> Result<T> {
        Err(Error::FileDescriptor(self))
    }
}

impl Error {
    pub const fn err<T>(self) -> Result<T> {
        Err(self)
    }
}

macro_rules! fd_flags {
    (
        $(
            $flag:ident = $value:expr
        ),*
        $(,)?
    ) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct FdFlags(c_int);

        impl FdFlags {
            $(
                pub const $flag: Self = Self($value);
            )*
        }
    };
}

fd_flags! {
    R_ONLY = libc::O_RDONLY,
    W_ONLY = libc::O_WRONLY,
    RW = libc::O_RDWR,
    ACCESS_MODE = libc::O_ACCMODE,
}

impl FdFlags {
    #[must_use]
    #[inline(always)]
    pub const fn from_bits(flags: c_int) -> Self {
        Self(flags)
    }

    #[must_use]
    #[inline(always)]
    pub const fn bits(self) -> c_int {
        self.0
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn has_all(self, flags: Self) -> bool {
        self.0 & flags.0 == flags.0
    }

    #[must_use]
    #[inline(always)]
    pub const fn has_any(self, flags: Self) -> bool {
        self.0 & flags.0 != 0
    }

    #[must_use]
    #[inline(always)]
    pub const fn has_none(self, flags: Self) -> bool {
        self.0 & flags.0 == 0
    }

    #[must_use]
    #[inline(always)]
    pub const fn or(self, flags: Self) -> Self {
        Self(self.0 | flags.0)
    }

    #[must_use]
    #[inline(always)]
    pub const fn and(self, flags: Self) -> Self {
        Self(self.0 & flags.0)
    }

    #[must_use]
    #[inline(always)]
    pub const fn xor(self, flags: Self) -> Self {
        Self(self.0 ^ flags.0)
    }

    #[must_use]
    #[inline(always)]
    pub const fn eq(self, flags: Self) -> bool {
        self.0 == flags.0
    }

    #[must_use]
    #[inline(always)]
    pub const fn ne(self, flags: Self) -> bool {
        self.0 != flags.0
    }

    #[must_use]
    #[inline(always)]
    pub const fn access_mode(self) -> Option<AccessMode> {
        Some(match self.and(Self::ACCESS_MODE) {
            Self::R_ONLY => AccessMode::ReadOnly,
            Self::W_ONLY => AccessMode::WriteOnly,
            Self::RW => AccessMode::ReadWrite,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccessMode {
    ReadOnly = 0,
    WriteOnly = 1,
    ReadWrite = 2,
}

pub struct FileDescriptor {
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
    pub fn from_fd(fd: c_int) -> Result<Self> {
        Ok(Self {
            fd: Errno::filter_result(unsafe { libc::fcntl(fd, libc::F_GETFD) })?,
        })
    }

    pub fn flags(&self) -> Result<FdFlags> {
        let flags = Errno::filter_result(unsafe { libc::fcntl(self.fd, libc::F_GETFL) })?;
        Ok(FdFlags(flags))
    }

    pub fn access_mode(&self) -> Result<AccessMode> {
        let flags = self.flags()?;
        Ok(match flags.and(FdFlags::ACCESS_MODE) {
            FdFlags::R_ONLY => AccessMode::ReadOnly,
            FdFlags::W_ONLY => AccessMode::WriteOnly,
            FdFlags::RW => AccessMode::ReadWrite,
            _ => unreachable!(),
        })
    }

    #[inline]
    pub fn is_reader(&self) -> Result<bool> {
        Ok(self.access_mode()? != AccessMode::WriteOnly)
    }

    #[inline]
    pub fn is_read_only(&self) -> Result<bool> {
        Ok(self.access_mode()? == AccessMode::ReadOnly)
    }

    #[inline]
    pub fn is_writer(&self) -> Result<bool> {
        Ok(self.access_mode()? != AccessMode::ReadOnly)
    }

    #[inline]
    pub fn is_write_only(&self) -> Result<bool> {
        Ok(self.access_mode()? == AccessMode::WriteOnly)
    }

    #[inline]
    pub fn is_read_write(&self) -> Result<bool> {
        Ok(self.access_mode()? == AccessMode::ReadWrite)
    }
    
    pub fn close(&mut self) -> Result {
        let result = unsafe { libc::fcntl(self.fd, libc::F_GETFD) };
        if result == -1 {
            return Ok(());
        }
        let close_result = unsafe { libc::close(self.fd) };
        if close_result == -1 {
            return Errno::get_err();
        }
        Ok(())
    }

    #[inline]
    pub fn is_open(&self) -> bool {
        let result = unsafe { libc::fcntl(self.fd, libc::F_GETFD) };
        return result != -1;
    }

    pub fn dup(&self) -> Result<Self> {
        Ok(Self {
            fd: Errno::filter_result(unsafe { libc::dup(self.fd) })?,
        })
    }

    pub fn dup2(&self, fd: c_int) -> Result<Self> {
        let result = unsafe { libc::fcntl(self.fd, libc::F_GETFD) };
        if result == -1 {
            return Err(Error::FileDescriptor(FileDescriptorError::InvalidOrClosed));
        }
        let result = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if result != -1 {
            return Err(Error::FileDescriptor(FileDescriptorError::AlreadyUsed));
        }
        debug_assert_eq!(
            fd,
            Errno::filter_result(unsafe { libc::dup2(self.fd, fd) })?,
            "dup2 new_fd not equal to returned fd.",
        );
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
    pub fn new(fd: FileDescriptor) -> Result<Self> {
        if !fd.is_open() {
            return FileDescriptorError::InvalidOrClosed.err();
        }
        if !fd.is_reader()? {
            return FileDescriptorError::NotAReader.err();
        }
        Ok(Self { fd })
    }

    pub fn close(&mut self) -> Result<()> {
        self.fd.close()
    }

    #[inline(always)]
    fn ensure_open(&self) -> Result<()> {
        if !self.fd.is_open() {
            return FileDescriptorError::InvalidOrClosed.err();
        }
        Ok(())
    }

    #[inline(always)]
    fn read_internal(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read_count = unsafe { libc::read(self.fd.fd, buf.as_mut_ptr().cast(), buf.len()) };
        if read_count < 0 {
            return Errno::get_err();
        }
        Ok(read_count as usize)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.ensure_open()?;
        self.read_internal(buf)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.ensure_open()?;
        let mut count = 0usize;
        while count < buf.len() {
            let read_len = self.read_internal(&mut buf[count..])?;
            if read_len == 0 {
                break;
            }
            count += read_len;
        }
        Ok(count)
    }
}

impl Writer {
    pub fn new(fd: FileDescriptor) -> Result<Self> {
        if !fd.is_writer()? {
            return FileDescriptorError::NotAReader.err();
        }
        Ok(Self { fd })
    }

    pub fn close(&mut self) -> Result<()> {
        self.fd.close()
    }

    #[inline(always)]
    pub fn ensure_open(&self) -> Result<()> {
        if !self.fd.is_open() {
            return FileDescriptorError::InvalidOrClosed.err();
        }
        Ok(())
    }

    fn write_internal(&mut self, buf: &[u8]) -> Result<usize> {
        let result = unsafe { libc::write(self.fd.fd, buf.as_ptr().cast(), buf.len()) };
        if result < 0 {
            return Errno::get_err();
        }
        Ok(result as usize)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.ensure_open()?;
        self.write_internal(buf)
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<usize> {
        self.ensure_open()?;
        let mut count = 0usize;
        while count < buf.len() {
            let write_len = self.write_internal(buf)?;
            if write_len == 0 {
                break;
            }
            count += write_len;
        }
        Ok(count)
    }
}

pub fn pipe(read_fd: Option<c_int>, write_fd: Option<c_int>) -> Result<(Reader, Writer)> {
    if read_fd.is_some() && read_fd == write_fd {
        return Err(Error::SameFileDescriptor);
    }
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

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Message {
    Cancel = 0,
    Shutdown = 1,
    Restart = 2,
    Error = 3,
}

impl Message {
    pub const CANCEL: u8 = Self::Cancel.as_u8();
    pub const SHUTDOWN: u8 = Self::Shutdown.as_u8();
    pub const RESTART: u8 = Self::Restart.as_u8();
    pub const ERROR: u8 = Self::Error.as_u8();

    #[must_use]
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Message> {
        match value {
            Self::CANCEL => Some(Self::Cancel),
            Self::SHUTDOWN => Some(Self::Shutdown),
            Self::RESTART => Some(Self::Restart),
            Self::ERROR => Some(Self::Error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipe_test() -> Result<()> {
        let (mut reader, mut writer) = pipe(Some(420), Some(69))?;

        assert_eq!(reader.fd.fd, 420);
        assert_eq!(writer.fd.fd, 69);
        
        let data = b"hello";
        
        assert_eq!(5, writer.write_all(data)?);

        let mut buf = [0u8; 5];

        assert_eq!(5, reader.read_exact(&mut buf)?);

        writer.close()?;
        
        assert_eq!(0, reader.read_exact(&mut buf)?);
        assert_eq!(0, reader.read_exact(&mut buf)?);

        assert_eq!(&buf, data);
        

        println!("Just to make sure...");

        Ok(())
    }
}
