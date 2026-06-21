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

macro_rules! fd_flags {
    (
        $(
            $vis:vis $flag:ident = $value:expr
        ),*
        $(,)?
    ) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct FdFlags(pub(crate) c_int);

        impl FdFlags {
            $(
                $vis const $flag: Self = Self($value);
            )*
        }
    };
}

fd_flags! {
    pub R_ONLY = libc::O_RDONLY,
    pub W_ONLY = libc::O_WRONLY,
    pub RW = libc::O_RDWR,
    pub ACCESS_MODE = libc::O_ACCMODE,
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
    /// Access is read only.
    ReadOnly = 0,
    /// Access is write only.
    WriteOnly = 1,
    /// Access is read and write.
    ReadWrite = 2,
}
