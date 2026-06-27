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


//=use
use crate::{
    fd::FileDescriptor,
    ffi::{
        read, read_exact,
        write, write_all,
    },
    error:: {
        Result,
        FdError, FdResult,
        ReadResult, WriteResult,
    },
};

/* Message System
// supervisor <- child
(reader, writer) = pipe();

match fork() {
    Parent => {
        close(writer);
        let mut restart_requested = false;
        let mut data_buffer = DataBuffer::new();
        'read_loop: loop {
            match read_message(reader) {
                Yield => continue,
                Restart => restart_requested = true,
                Cancel => restart_requested = false,
                Data => {
                    // Begin data reading loop
                    'data_loop: loop {
                        // packet_len must be correct for the length being written.
                        let packet_len = read_usize(reader);
                        // 0 is used as a sentinel to mean "no more to read".
                        // The `0` sentinel MUST be sent when there is no more data being sent.
                        if packet_len == 0 {
                            break;
                        }
                        data_buffer.read_into(reader, packet_len);
                    }
                }
            }
        }
    },
    Child => {
        close(reader);
    },
}
*/

//=code

//=types


/// A message enum that is used to communicate with a supervisor process.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Msg {
    // NOTE: The discriminants are important.
    /// Essentially does nothing besides making the read block yield.
    Yield = 0x00,
    /// Request a restart.
    Restart = 0x01,
    /// Request to cancel the restart request.
    Cancel = 0x02,
    /// Marks the start and end of data.
    ///
    /// When the `Data` message is received for the first time, the parent
    /// will go into data reading mode. It will poll the message and data
    /// readers. It will read data from the data reader until it receives
    /// another Data message from the message reader. The second `Data`
    /// message signifies that the data is finished sending.
    BeginData = 0x03,
    ResetData = 0x04,
    FreeData = 0x05,
    // TODO: Update MAX_KNOWN and from_u8() whenever new variants are added.
    Value(u8),
}

pub struct MsgReader {
    fd: FileDescriptor,
}

pub struct MsgWriter {
    fd: FileDescriptor,
}

//=impls

impl Msg {
    const YIELD: u8 = 0x00;
    const RESTART: u8 = 0x01;
    const CANCEL: u8 = 0x02;
    const DATA: u8 = 0x03;
    const RESET: u8 = 0x04;
    const FREE: u8 = 0x05;
    pub const MAX_KNOWN: u8 = 0x05;
    const CUSTOM_START: u8 = Self::MAX_KNOWN + 1;
    pub const fn from_u8(value: u8) -> Self {
        const ALL: [Msg; 6] = [
            Msg::Yield,
            Msg::Restart,
            Msg::Cancel,
            Msg::BeginData,
            Msg::ResetData,
            Msg::FreeData,
        ];
        let index = value as usize;
        if index < ALL.len() {
            ALL[index]
        } else {
            Self::Value(value)
        }
    }

    #[must_use]
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        match self {
            Msg::Yield => Self::YIELD,
            Msg::Restart => Self::RESTART,
            Msg::Cancel => Self::CANCEL,
            Msg::BeginData => Self::DATA,
            Msg::ResetData => Self::RESET,
            Msg::FreeData => Self::FREE,
            Msg::Value(unk) => unk as u8,
        }
    }
}

impl MsgReader {
    #[must_use]
    #[inline(always)]
    pub fn new(fd: FileDescriptor) -> FdResult<Self> {
        if !fd.is_reader()? {
            return Err(FdError::NotAReader);
        }
        Ok(Self { fd })
    }

    #[inline(always)]
    pub fn read_data(&mut self, buf: &mut [u8]) -> ReadResult<usize> {
        unsafe { read_exact(self.fd.fd, buf) }
    }

    #[inline]
    pub fn read_msg(&mut self) -> ReadResult<Option<Msg>> {
        let mut buf = [0u8; 1];
        let read_len = self.read_data(&mut buf)?;
        if read_len == 0 {
            return Ok(None)
        }
        Ok(Some(Msg::from_u8(buf[0])))
    }

    #[inline]
    pub fn read_usize(&mut self) -> ReadResult<Option<usize>> {
        let mut buf = [0u8; size_of::<usize>()];
        let read_len = self.read_data(&mut buf)?;
        if read_len < buf.len() {
            return Ok(None)
        }
        Ok(Some(usize::from_ne_bytes(buf)))
    }
}

impl MsgWriter {
    #[must_use]
    #[inline(always)]
    pub fn new(fd: FileDescriptor) -> FdResult<Self> {
        if !fd.is_writer()? {
            return Err(FdError::NotAWriter);
        }
        Ok(Self { fd })
    }

    #[inline(always)]
    pub fn write_data(&mut self, data: &[u8]) -> WriteResult<usize> {
        unsafe { write_all(self.fd.fd, data) }
    }

    #[inline]
    pub fn write_msg(&mut self, msg: Msg) -> WriteResult<bool> {
        let buf = [msg.as_u8(); 1];
        let write_len = self.write_data(&buf)?;
        Ok(write_len == 1)
    }

    #[inline]
    pub fn write_usize(&mut self, value: usize) -> WriteResult<bool> {
        let buf = value.to_ne_bytes();
        let write_len = self.write_data(&buf)?;
        Ok(write_len == buf.len())
    }
}
