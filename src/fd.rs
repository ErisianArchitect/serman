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
    c_int,
};

use crate::error::{
    FdResult,
    FdError,
    DupResult,
    DupError,
};

use crate::fd_flags::{FdFlags, AccessMode};
use crate::ffi;

//= src/fd.rs::FileDescriptor
#[repr(transparent)]
pub struct FileDescriptor {
    fd: c_int,
}

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        match self.close() {
            Ok(()) => {},
            Err(_) => (/* Ignore Error */),
        }
    }
}

impl FileDescriptor {
    /// Creates a new [FileDescriptor] from its raw value.
    /// 
    /// File descriptor must be both open and valid.
    #[must_use]
    #[inline(always)]
    pub fn from_fd(fd: c_int) -> FdResult<Self> {
        unsafe { ffi::fd_flags(fd)? };
        Ok(Self {
            fd,
        })
    }

    /// Get the flags for this file descriptor.
    pub fn flags(&self) -> FdResult<FdFlags> {
        let flags = unsafe { ffi::fd_flags(self.fd)? };
        Ok(FdFlags(flags))
    }

    /// Get the access mode for this file descriptor.
    pub fn access_mode(&self) -> FdResult<AccessMode> {
        let flags = self.flags()?;
        Ok(match flags.and(FdFlags::ACCESS_MODE) {
            FdFlags::R_ONLY => AccessMode::ReadOnly,
            FdFlags::W_ONLY => AccessMode::WriteOnly,
            FdFlags::RW => AccessMode::ReadWrite,
            _ => unreachable!(),
        })
    }

    #[inline]
    pub fn is_reader(&self) -> FdResult<bool> {
        Ok(self.access_mode()? != AccessMode::WriteOnly)
    }

    #[inline]
    pub fn is_read_only(&self) -> FdResult<bool> {
        Ok(self.access_mode()? == AccessMode::ReadOnly)
    }

    #[inline]
    pub fn is_writer(&self) -> FdResult<bool> {
        Ok(self.access_mode()? != AccessMode::ReadOnly)
    }

    #[inline]
    pub fn is_write_only(&self) -> FdResult<bool> {
        Ok(self.access_mode()? == AccessMode::WriteOnly)
    }

    #[inline]
    pub fn is_read_write(&self) -> FdResult<bool> {
        Ok(self.access_mode()? == AccessMode::ReadWrite)
    }
    
    pub fn close(&mut self) -> FdResult<()> {
        let close_result = unsafe { libc::close(self.fd) };
        if close_result == -1 {
            return FdError::get_err();
        }
        Ok(())
    }

    #[inline]
    pub fn is_open(&self) -> bool {
        let result = unsafe { libc::fcntl(self.fd, libc::F_GETFD) };
        return result != -1;
    }

    pub fn dup(&self) -> DupResult<Self> {
        let dup_result = unsafe { libc::dup(self.fd) };
        if dup_result == -1 {
            return DupError::get_err();
        }
        Ok(Self {
            fd: dup_result,
        })
    }

    pub fn dup2(&self, fd: c_int) -> DupResult<Self> {
        let dup_result = unsafe { libc::dup2(self.fd, fd) };
        if dup_result == -1 {
            return DupError::get_err();
        }
        // this check likely isn't necessary, but I thought "why not?"
        debug_assert_eq!(
            fd,
            dup_result,
            "dup2 new_fd not equal to returned fd.",
        );
        Ok(Self { fd })
    }

    pub fn rebind(&mut self, fd: c_int) -> DupResult {
        let new = self.dup2(fd)?;
        *self = new;
        Ok(())
    }
}
