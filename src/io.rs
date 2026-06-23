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

use crate::{
    error::{
        ReadResult, ReadError,
        WriteResult, WriteError,
        FdResult, FdError,
    },
    fd::FileDescriptor,
    fd_flags::{AccessMode, FdFlags},
    ffi,
};



pub struct Reader {
     pub(crate) fd: FileDescriptor,
}

pub struct Writer {
    pub(crate) fd: FileDescriptor,
}

impl Reader {
    #[inline]
    pub fn from_fd(fd: c_int) -> FdResult<Self> {
        // Here we skip from_fd because we're just going to check the
        // flags in the next call.
        let fd = FileDescriptor { fd };
        if !fd.is_reader()? {
            return Err(FdError::NotAReader);
        }
        Ok(Self { fd })
    }
    
    #[inline(always)]
    pub fn read(&mut self, buf: &mut [u8]) -> ReadResult<usize> {
        unsafe { ffi::read(self.fd.fd, buf) }
    }

    #[inline(always)]
    pub fn read_exact(&mut self, buf: &mut [u8]) -> ReadResult<usize> {
        unsafe { ffi::read_exact(self.fd.fd, buf) }
    }
}

impl Writer {
    #[inline]
    pub fn from_fd(fd: c_int) -> FdResult<Self> {
        // Here we skip from_fd because we're just going to check the
        // flags in the next call.
        let fd = FileDescriptor { fd };
        if !fd.is_writer()? {
            return Err(FdError::NotAWriter);
        }
        Ok(Self { fd })
    }

    #[inline(always)]
    pub fn write(&mut self, buf: &[u8]) -> WriteResult<usize> {
        unsafe { ffi::write(self.fd.fd, buf) }
    }

    #[inline(always)]
    pub fn write_all(&mut self, buf: &[u8]) -> WriteResult<usize> {
        unsafe { ffi::write_all(self.fd.fd, buf) }
    }
}
