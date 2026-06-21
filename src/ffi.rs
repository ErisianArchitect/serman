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



use libc::{
    // Types
    c_int, c_uint, c_short,
    pid_t,
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
    DupError,
    DupResult,
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

//-start

// TODO: Do something with this.
/// Raw, low-level file descriptor.
#[repr(transparent)]
pub struct FileDesc {
    fd: c_int,
}

#[inline]
pub unsafe fn dup(fd: c_int) -> DupResult<c_int> {
    let dup_result = unsafe { libc::dup(fd) };
    if dup_result == -1 {
        return DupError::get_err();
    }
    Ok(dup_result)
}

#[inline]
pub unsafe fn dup2(src: c_int, dst: c_int) -> DupResult<c_int> {
    let dup_result = unsafe { libc::dup2(src, dst) };
    if dup_result == -1 {
        return DupError::get_err();
    }
    Ok(dup_result)
}

/// Used to obtain the reader and writer fds from a pipe call.
#[repr(C)]
#[cfg_attr(any(target_arch = "avr", target_arch = "msp430"), repr(align(4)))]
#[cfg_attr(not(any(target_arch = "avr", target_arch = "msp430")), repr(align(8)))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipeFds {
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
pub unsafe fn pipe() -> PipeResult<PipeFds> {
    let mut fds = PipeFds::zeroed();
    let pipe_result = unsafe { libc::pipe(fds.as_mut_fds()) };
    if pipe_result == -1 {
        return PipeError::get_err();
    }
    Ok(fds)
}

// This function is specifically written for writing to unix pipes. It's just a wrapper around libc's `write`, so it
// will likely work just fine for other writing purpose that require libc's write. It just gives you
// `Result<count_written>` instead of arcane error sentinels.
#[inline]
pub unsafe fn write(fd: c_int, buf: &[u8]) -> WriteResult<usize> {
    let write_count = unsafe { libc::write(fd, buf.as_ptr().cast(), buf.len()) };
    if write_count == -1 {
        return WriteError::get_err();
    }
    Ok(write_count as usize)
}

// This was written for writing to unix pipes. There are no guarantees with other file descriptors, or really even any guarantees at all!
#[inline]
pub unsafe fn write_all(fd: c_int, buf: &[u8]) -> WriteResult<usize> {
    let mut count = 0usize;
    while count < buf.len() {
        // to safely get a subslice of buf before casting it to a pointer.
        let rest = &buf[count..buf.len()];
        // Above, there is an implementation of `write` that is suitable as a wrapper for `libc's write`, but there's one less branch
        // if it's written without the wrapper here.
        match unsafe { libc::write(fd, rest.as_ptr().cast(), rest.len()) } {
            -1 => return WriteError::get_err(),
            // This means that nothing was written. I don't really know what this means, sorry. the manual says that it's unspecified
            // (at least in terms of writing to a pipe, which this function was written for)
            0 => break,
            len => count += len as usize,
        }
    }
    Ok(count)
}

#[inline]
pub unsafe fn read(fd: c_int, buf: &mut [u8]) -> ReadResult<usize> {
    let read_count = unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) };
    if read_count == -1 {
        return ReadError::get_err();
    }
    Ok(read_count as usize)
}

#[inline]
pub unsafe fn read_exact(fd: c_int, buf: &mut [u8]) -> ReadResult<usize> {
    let mut count = 0usize;
    while count < buf.len() {
        let end = buf.len();
        let rest = &mut buf[count..end];
        match unsafe { libc::read(fd, rest.as_mut_ptr().cast(), rest.len()) } {
            -1 => return ReadError::get_err(),
             0 => break,
             len => count += len as usize,
        }
    }
    Ok(count)
}

// TODO: If you change the way generics work on Fork, make sure to update any code that might rely on the current implementation.
/// Represents a value from the branch of a fork.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fork<P = (), C = P> {
    /// The parent branch of the fork with the child's PID.
    Parent(P),
    /// The child branch of the fork with the parent's PID.
    Child(C),
}

#[inline]
pub unsafe fn fork() -> ForkResult<Fork<pid_t>> {
    let parent_pid: pid_t = unsafe { libc::getpid() };
    let fork_result = unsafe { libc::fork() };
    match fork_result {
        // There was an error, obviously.
        -1 => return ForkError::get_err(),
        // child branch
        0 => Ok(Fork::Child(parent_pid)),
        // parent branch
        child_id => Ok(Fork::Parent(child_id)),
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WaitStatus {
    status: c_int,
}

impl WaitStatus {
    #[must_use]
    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut c_int {
        self as *mut _ as _
    }

    // TODO: Accessors for WaitStatus.
}

#[inline]
pub unsafe fn waitpid(pid: pid_t) -> WaitResult<(pid_t, WaitStatus)> {
    let mut status = WaitStatus { status: 0 };
    let wait_result = unsafe { libc::waitpid(pid, status.as_mut_ptr(), 0) };
    if wait_result == -1 {
        return WaitError::get_err();
    }
    Ok((wait_result, status))
}

/// Get file status flags. (`libc::fcntl(fd, libc::F_GETFL)`)
#[inline]
pub unsafe fn fd_status(fd: c_int) -> FdResult<c_int> {
    let result = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if result == -1 {
        return FdError::get_err();
    }
    Ok(result)
}

/// Get the file descriptor flags. (`libc::fcntl(fd, libc::F_GETFD)`)
#[inline]
pub unsafe fn fd_flags(fd: c_int) -> FdResult<c_int> {
    let result = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if result == -1 {
        return FdError::get_err();
    }
    Ok(result)
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PollEvents(c_short);

macro_rules! poll_events {
    (
        $(
            $event:ident
        ),*
        $(,)?
    ) => {
        $(
            pub const $event: Self = Self(libc::$event);
        )*
    };
}

impl PollEvents {
    poll_events!(
        POLLOUT,
        POLLIN,
        POLLPRI,
        POLLRDHUP,
        POLLERR,
        POLLHUP,
        POLLNVAL,
    );

    #[must_use]
    #[inline(always)]
    pub const fn new() -> Self {
        Self(0);
    }

    #[must_use]
    #[inline(always)]
    pub const fn add(&mut self, events: Self) {
        self.0 |= events.0;
    }

    #[must_use]
    #[inline(always)]
    pub const fn remove(&mut self, events: Self) {
        self.0 &= !events.0;
    }

    #[must_use]
    #[inline(always)]
    pub const fn or(self, events: Self) -> Self {
        Self(self.0 | events.0)
    }

    #[must_use]
    #[inline(always)]
    pub const fn and(self, events: Self) -> Self {
        Self(self.0 & events.0)
    }

    #[must_use]
    #[inline(always)]
    pub const fn xor(self, events: Self) -> Self {
        Self(self.0 ^ events.0)
    }

    #[must_use]
    #[inline(always)]
    pub const fn union(events: &[Self]) -> Self {
        let mut builder = Self::new();
        let mut index = 0usize;
        while index < events.len() {
            builder.add(events[index]);
            index += 1;
        }
        builder
    }

    #[must_use]
    #[inline(always)]
    pub const fn has_all(self, events: Self) -> bool {
        self.0 & events.0 == events.0
    }

    #[must_use]
    #[inline(always)]
    pub const fn has_any(self, events: Self) -> bool {
        self.0 & events.0 != 0
    }

    #[must_use]
    #[inline(always)]
    pub const fn has_none(self, events: Self) -> bool {
        self.0 & events.0 == 0
    }
}

#[repr(C)]
pub struct PollFd {
    fd: c_int,
    events: PollEvents,
    revents: PollEvents,
}

impl PollFd {
    #[must_use]
    #[inline(always)]
    pub const fn new(fd: c_int, events: PollEvents) -> Self {
        Self {
            fd,
            events,
            revents: PollEvents::new(),
        }
    }
}

/// Note: A `timeout_ms` of `0` means return instantly, even if nothing is ready. `-1` means wait forever.
#[inline]
pub unsafe fn poll(fds: &mut [PollFd], timeout_ms: c_int) -> PollResult<usize>{
    let poll_result = unsafe { libc::poll(fds.as_mut_ptr().cast(), fds.len() as u64, timeout_ms) };
    if poll_result == -1 {
        return PollError::get_err();
    }
    Ok(poll_result as usize)
}
