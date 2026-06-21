//  SPDX-License-Identifier: Apache-2.0
//  Copyright © 2026-present Ada F. <https://github.com/ErisianArchitect>
//  
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//  
//      http://www.apache.org/licenses/LICENSE-2.0
//  
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
//:---[END-HEADER]---

mod entry;
mod error;
mod fd;
mod fd_flags;
mod ffi;
mod ref_count;
mod util;

use parking_lot::Mutex;
use std::{time::SystemTime};
use ::core::{
    ptr::NonNull,
    alloc::Layout,
};

use ref_count::RefCounter32;

use libc::{
    c_int,
};

use fd_flags::{AccessMode, FdFlags};

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
    #[error("Unrecognized child signal byte: 0x{0:X}")]
    UnrecognizedChildSignal(u8),
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

    #[must_use]
    #[inline(always)]
    pub fn raw_fd(&self) -> c_int {
        self.fd.fd
    }

    #[must_use]
    #[inline(always)]
    pub fn fd(&self) -> &FileDescriptor {
        &self.fd
    }

    #[inline]
    pub fn dup(&self) -> Result<Self> {
        Ok(Self {
            fd: self.fd.dup()?,
        })
    }

    #[inline]
    pub fn dup2(&self, fd: c_int) -> Result<Self> {
        Ok(Self {
            fd: self.fd.dup2(fd)?,
        })
    }

    #[inline]
    pub fn rebind(&mut self, new_fd: c_int) -> Result {
        self.fd.rebind(new_fd)
    }

    pub fn close(&mut self) -> Result<()> {
        self.fd.close()
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
        self.read_internal(buf)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<usize> {
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

    #[must_use]
    #[inline(always)]
    pub fn raw_fd(&self) -> c_int {
        self.fd.fd
    }

    #[must_use]
    #[inline(always)]
    pub fn fd(&self) -> &FileDescriptor {
        &self.fd
    }

    #[inline]
    pub fn dup(&self) -> Result<Self> {
        Ok(Self {
            fd: self.fd.dup()?,
        })
    }

    #[inline]
    pub fn dup2(&self, fd: c_int) -> Result<Self> {
        Ok(Self {
            fd: self.fd.dup2(fd)?,
        })
    }

    #[inline]
    pub fn rebind(&mut self, new_fd: c_int) -> Result {
        self.fd.rebind(new_fd)
    }

    pub fn close(&mut self) -> Result<()> {
        self.fd.close()
    }

    fn write_internal(&mut self, buf: &[u8]) -> Result<usize> {
        let result = unsafe { libc::write(self.fd.fd, buf.as_ptr().cast(), buf.len()) };
        if result < 0 {
            return Errno::get_err();
        }
        Ok(result as usize)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.write_internal(buf)
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<usize> {
        let mut count = 0usize;
        while count < buf.len() {
            let write_len = self.write_internal(&buf[count..])?;
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
pub enum Message {
    Cancel = 0,
    Restart = 1,
    StartData = 2,
    EndData = 3,
    ClearData = 4,
    // TODO: If you add any more messages, make sure to update Message::MAX to the new maximum value.
}

impl Message {
    pub const CANCEL: u8 = Self::Cancel.as_u8();
    pub const RESTART: u8 = Self::Restart.as_u8();
    pub const START_DATA: u8 = Self::StartData.as_u8();
    pub const END_DATA: u8 = Self::EndData.as_u8();
    pub const CLEAR_DATA: u8 = Self::ClearData.as_u8();
    const MAX: u8 = Self::ClearData.as_u8();

    #[must_use]
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Message> {
        if value > Self::MAX {
            return None;
        }
        Some(unsafe { ::core::mem::transmute(value) })
    }
}

pub struct MsgSend {
    writer: Writer,
}

pub struct MsgRecv {
    reader: Reader,
}

impl MsgSend {
    pub fn send(&mut self, msg: Message) -> Result<bool> {
        let write_len = self.writer.write_all(&[msg.as_u8()])?;
        Ok(write_len != 0)
    }

    pub fn restart(&mut self) -> Result<bool> {
        self.send(Message::Restart)
    }

    pub fn cancel(&mut self) -> Result<bool> {
        self.send(Message::Cancel)
    }
}

impl MsgRecv {
    pub fn recv(&mut self) -> Result<Option<Message>> {
        let mut buf = [0u8; 1];
        let read_len = self.reader.read_exact(&mut buf)?;
        if read_len == 0 {
            return Ok(None);
        }
        Ok(Message::from_u8(buf[0]))
    }

    // pub fn poll<const COUNT: usize>(receivers: &[&MsgRecv; COUNT], &mut [bool; COUNT]) -> 
}

// TODO: Use this when implementing the session data system.
#[repr(C, align(8))]
struct ChildSenders {
    msg: MsgSend,
    data: Writer,
}

#[repr(C)]
struct ContextInner {
    sender: Mutex<MsgSend>,
    ref_count: RefCounter32,
    restart_count: u64,
    entry_time: SystemTime,
}

#[repr(transparent)]
pub struct ForkContext {
    ptr: NonNull<ContextInner>,
}

impl ContextInner {
    const LAYOUT: Layout = Layout::new::<Self>();

    unsafe fn alloc(value: Self) -> NonNull<Self> {
        unsafe {
            let ptr = std::alloc::alloc(Self::LAYOUT).cast::<Self>();
            let Some(ptr) = NonNull::new(ptr) else {
                std::alloc::handle_alloc_error(Self::LAYOUT);
            };
            ptr.write(value);
            ptr
        }
    }

    unsafe fn dealloc(ptr: NonNull<Self>) {
        unsafe {
            std::alloc::dealloc(ptr.as_ptr().cast(), Self::LAYOUT);
        }
    }
}

impl ForkContext {
    #[must_use]
    #[inline(always)]
    fn get_ref(&self) -> &ContextInner {
        unsafe { self.ptr.as_ref() }
    }

    fn new(sender: MsgSend, restart_count: u64, entry_time: SystemTime) -> Self {
        let inner = ContextInner {
            sender: Mutex::new(sender),
            ref_count: RefCounter32::new(1),
            restart_count,
            entry_time,
        };
        let ptr = unsafe { ContextInner::alloc(inner) };
        Self {
            ptr
        }
    }

    /// The ID associated with this fork context.
    /// 
    /// `0` would be the first fork, and all subsequent forks are numbered sequentially.
    /// 
    /// This can be considered to be the number of times that the process has been restarted.
    #[must_use]
    #[inline(always)]
    pub fn restart_id(&self) -> u64 {
        self.get_ref().restart_count
    }

    /// The time that the entry function was initially called in the root process before any
    /// child process has been forked.
    #[must_use]
    #[inline(always)]
    pub fn entry_time(&self) -> SystemTime {
        self.get_ref().entry_time
    }

    /// Send a message to the supervisor process.
    #[inline(always)]
    pub fn send(&self, msg: Message) -> Result<bool> {
        let mut sender = self.get_ref().sender.lock();
        sender.send(msg)
    }

    /// Send a restart request to the supervisor process.
    /// 
    /// This will not cause the process to restart immediately, instead it will restart
    /// on process exit. You can call [ForkContext::cancel] to cancel the restart and proceed
    /// to exit from the parent process.
    #[inline(always)]
    pub fn restart(&self) -> Result<bool> {
        self.send(Message::Restart)
    }

    /// Cancel a previously sent restart request.
    #[inline(always)]
    pub fn cancel(&self) -> Result<bool> {
        self.send(Message::Cancel)
    }
}

impl Drop for ForkContext {
    fn drop(&mut self) {
        if let Ok(0) = self.get_ref().ref_count.decrement() {
            unsafe { ContextInner::dealloc(self.ptr); }
        }
    }
}

impl Clone for ForkContext {
    fn clone(&self) -> Self {
        self.get_ref().ref_count.increment();
        Self {
            ptr: self.ptr,
        }
    }
}

#[must_use]
pub enum EntryResult<C> {
    Parent(Result<()>),
    Child(C),
}

enum ParentResult {
    Restart,
    Exit,
}

// TODO: Placeholder type, do not keep.
type Placeholder = ();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SignalAction {
    Ignore,
    Restart,
    Exit(u8),
    Filter(c_int),
}

impl SignalAction {
    pub fn ignore(_: c_int) -> SignalAction {
        SignalAction::Ignore
    }

    pub fn restart(_: c_int) -> SignalAction {
        SignalAction::Restart
    }

    pub fn restart_if<const SIGNAL: c_int>(signal: c_int) -> SignalAction {
        if signal == SIGNAL {
            SignalAction::Restart
        } else {
            SignalAction::Filter(signal)
        }
    }

    pub fn exit<const EXIT: u8>(_: c_int) -> SignalAction {
        SignalAction::Exit(EXIT)
    }

    pub fn exit_if<const SIGNAL: c_int, const EXIT_CODE: u8>(signal: c_int) -> SignalAction {
        if signal == SIGNAL {
            SignalAction::Exit(EXIT_CODE)
        } else {
            SignalAction::Filter(signal)
        }
    }

    pub fn exit_success(_: c_int) -> SignalAction {
        SignalAction::Exit(0)
    }

    pub fn exit_failure(_: c_int) -> SignalAction {
        SignalAction::Exit(1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExitAction {
    Restart,
    Exit(u8),
    Filter(u8),
}

impl ExitAction {
    pub fn restart(_: u8) -> ExitAction {
        ExitAction::Restart
    }

    pub fn exit<const EXIT: u8>(_: u8) -> ExitAction {
        ExitAction::Exit(EXIT)
    }

    pub fn exit_success(_: u8) -> ExitAction {
        ExitAction::Exit(0)
    }

    pub fn exit_failure(_: u8) -> ExitAction {
        ExitAction::Exit(1)
    }
}

#[repr(C)]
pub struct Entry<R: Sized + 'static> {
    main: fn(ForkContext) -> R,
    exit_signal_handler: fn(c_int) -> SignalAction,
    restart_signal_handler: fn(c_int) -> SignalAction,
    exit_handler: fn(u8) -> ExitAction,
    restart_handler: fn(u8) -> ExitAction,
}

impl<R: Sized + 'static> Entry<R> {
    #[must_use]
    #[inline(always)]
    pub const fn new(main: fn(ForkContext) -> R) -> Self {
        Self {
            main,
            exit_signal_handler: SignalAction::Filter,
            restart_signal_handler: SignalAction::Filter,
            exit_handler: ExitAction::Filter,
            restart_handler: ExitAction::Filter,
        }
    }

    #[must_use]
    #[inline(always)]
    pub const fn exit_signal_handler(mut self, handler: fn(c_int) -> SignalAction) -> Self {
        self.exit_signal_handler = handler;
        self
    }

    #[must_use]
    #[inline(always)]
    pub const fn restart_signal_handler(mut self, handler: fn(c_int) -> SignalAction) -> Self {
        self.restart_signal_handler = handler;
        self
    }

    #[must_use]
    #[inline(always)]
    pub const fn exit_handler(mut self, handler: fn(u8) -> ExitAction) -> Self {
        self.exit_handler = handler;
        self
    }

    #[must_use]
    #[inline(always)]
    pub const fn restart_handler(mut self, handler: fn(u8) -> ExitAction) -> Self {
        self.restart_handler = handler;
        self
    }

    #[must_use]
    #[inline(always)]
    fn handle_exit_signal(&self, signal: c_int) -> SignalAction {
        (self.exit_signal_handler)(signal)
    }

    #[must_use]
    #[inline(always)]
    fn handle_restart_signal(&self, signal: c_int) -> SignalAction {
        (self.restart_signal_handler)(signal)
    }

    #[must_use]
    #[inline(always)]
    fn handle_exit(&self, exit_code: u8) -> ExitAction {
        (self.exit_handler)(exit_code)
    }

    fn handle_restart(&self, exit_code: u8) -> ExitAction {
        (self.restart_handler)(exit_code)
    }

    pub fn run(self) -> EntryResult<R> {
        todo!()
    }
}

fn foo() {
    fn foo_main(ctx: ForkContext) -> Result<()> {
        Ok(())
    }
    let result = Entry::new(foo_main)
        .exit_handler(|code| {
            if code != 0 {
                ExitAction::Restart
            } else {
                ExitAction::Exit(0)
            }
        }).run();
}

pub unsafe fn entry<R>(main: fn(ForkContext) -> R) -> EntryResult<R> {
    let entry_time = SystemTime::now();
    let mut restart_count = 0u64;
    'fork_loop: loop {
        let (reader, writer) = match pipe(None, None) {
            Ok(pair) => pair,
            Err(err) => return EntryResult::Parent(Err(err)),
        };
        let pid_result = unsafe { libc::fork() };
        if pid_result == -1 {
            return EntryResult::Parent(Errno::get_err());
        }
        match pid_result {
            // child
            0 => {
                return EntryResult::Child((move || {
                    drop(reader);
                    let ctx = ForkContext::new(
                        MsgSend { writer },
                        restart_count,
                        entry_time,
                    );
                    main(ctx)
                })());
            }
            // parent
            pid => {
                let result = (move || {
                    drop(writer);
                    // TODO: Handle signals within the parent.
                    let mut receiver = MsgRecv { reader };
                    let mut exit_result = ParentResult::Exit;
                    'receive_loop: loop {
                        match receiver.recv()? {
                            Some(Message::Cancel) => exit_result = ParentResult::Exit,
                            Some(Message::Restart) => exit_result = ParentResult::Restart,
                            Some(Message::StartData) => {
                                
                            },
                            Some(Message::EndData) => {},
                            Some(Message::ClearData) => {},
                            None => break 'receive_loop,
                        }
                    }
                    let mut status: c_int = 0;
                    let wait_result = unsafe { libc::waitpid(pid, &mut status, 0) };
                    if wait_result == -1 {
                        return Errno::get_err();
                    }
                    let exit_status = if unsafe { libc::WIFEXITED(status) } {
                        Some(libc::WEXITSTATUS(status) as u8)
                    } else {
                        None
                    };
                    Ok(exit_result)
                })();
                match result {
                    Ok(ParentResult::Restart) => {
                        restart_count += 1;
                        continue 'fork_loop;
                    },
                    Ok(ParentResult::Exit) => return EntryResult::Parent(Ok(())),
                    Err(err) => return EntryResult::Parent(Err(err)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct Fail<T>(T);
    pub struct Pass<T>(T);

    impl<T: std::fmt::Display> std::fmt::Display for Fail<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("\x1b[38;2;255;75;75m")?;
            self.0.fmt(f)?;
            f.write_str("\x1b[39m")
        }
    }

    impl<T: std::fmt::Display> std::fmt::Display for Pass<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("\x1b[38;2;0;185;45m")?;
            self.0.fmt(f)?;
            f.write_str("\x1b[39m")
        }
    }

    #[test]
    fn pipe_test() -> Result<()> {
        let (mut reader, mut writer) = pipe(Some(420), Some(69))?;

        assert_eq!(reader.raw_fd(), 420);
        assert_eq!(writer.raw_fd(), 69);
        
        let data = b"hello";
        
        assert_eq!(5, writer.write_all(data)?);

        let mut buf = [0u8; 5];

        assert_eq!(5, reader.read_exact(&mut buf)?);

        writer.close()?;
        
        assert_eq!(0, reader.read_exact(&mut buf)?);

        assert_eq!(&buf, data);
        

        println!("{}: Everything works.", Pass("Success"));

        Ok(())
    }
}
