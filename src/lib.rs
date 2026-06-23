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

mod fd;
mod fd_flags;
mod ref_count;
mod util;

pub mod data;
pub mod messaging;
pub mod entry;
pub mod error;
pub mod io;
pub mod ffi;

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

use error::{Result, Error};
use fd::{FileDescriptor};
use fd_flags::{AccessMode, FdFlags};

#[cfg(not(unix))]
compile_error!("This library is for unix systems, and the target is not a unix system.");
