// SPDX-License-Identifier: Apache-2.0
// Copyright © 2026-present Ada F. <https://github.com/ErisianArchitect>
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use ::core::{
    ptr::NonNull,
};
use std::{
    alloc::{
        self,
        Layout,
    },
};

/// Align the target capacity to the growth factor.
fn grow_capacity(target_capacity: usize) -> usize {
    // Grow to next power of 2 until 4096, then grow by 4096.
    if target_capacity >= 4096 {
        target_capacity.next_multiple_of(4096)
    } else {
        target_capacity.max(64usize).next_power_of_two()
    }
}

#[repr(C)]
pub struct DataBuffer {
    ptr: NonNull<u8>,
    len: usize,
    capacity: usize,
}

impl Drop for DataBuffer {
    fn drop(&mut self) {
        // 0 capacity means unallocated.
        if self.capacity == 0 {
            return;
        }
        let layout = self.layout();
        unsafe { alloc::dealloc(self.ptr.as_ptr(), layout); }
    }
}

impl DataBuffer {
    #[must_use]
    #[inline(always)]
    fn layout(&self) -> Layout {
        Layout::array::<u8>(self.capacity).unwrap()
    }

    #[must_use]
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            capacity: 0,
        }
    }

    #[must_use]
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    #[must_use]
    #[inline(always)]
    pub(crate) fn full_slice(&self) -> &[u8] {
        unsafe {
            ::core::slice::from_raw_parts(self.ptr.as_ptr(), self.capacity)
        }
    }

    #[must_use]
    #[inline(always)]
    pub(crate) fn full_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            ::core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.capacity)
        }
    }

    #[must_use]
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            ::core::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }

    #[must_use]
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            ::core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }

    fn ensure_capacity(&mut self, capacity: usize) {
        if self.capacity >= capacity {
            return;
        }
        let grown_capacity = grow_capacity(capacity);
        if self.capacity == 0 {
            let layout = Layout::array::<u8>(grown_capacity).unwrap();
            let new_ptr = unsafe { alloc::alloc(layout) };
            let Some(ptr) = NonNull::new(new_ptr) else {
                alloc::handle_alloc_error(layout);
            };
            self.ptr = ptr;
            self.capacity = grown_capacity;
            return;
        }
        let old_layout = self.layout();
        let new_ptr = unsafe { alloc::realloc(self.ptr.as_ptr(), old_layout, grown_capacity) };
        let Some(ptr) = NonNull::new(new_ptr) else {
            alloc::handle_alloc_error(Layout::array::<u8>(grown_capacity).unwrap());
        };
        self.ptr = ptr;
        self.capacity = grown_capacity;
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Deallocate the memory owned by the data buffer. This will leave
    /// the data buffer in a valid state with a capacity and length of 0.
    #[inline]
    pub(crate) fn dealloc(&mut self) {
        let layout = self.layout();
        unsafe { alloc::dealloc(self.ptr.as_ptr(), layout) };
        self.capacity = 0;
        self.len = 0;
        self.ptr = NonNull::dangling();
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) {
        let new_len = self.len + bytes.len();
        self.ensure_capacity(new_len);
        let range = self.len..(self.len + bytes.len());
        let buf = &mut self.full_slice_mut()[range];
        buf.copy_from_slice(bytes);
        self.len = new_len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_data_buffer() {
        let mut data = DataBuffer::new();
        const DATA: &'static [u8] = b"The quick brown fox jumps over the lazy dog.";
        let mut check: Vec<u8> = Vec::new();
        check.extend(DATA);
        data.push_bytes(DATA);
        let buf = data.as_slice();
        assert_eq!(buf, DATA);
        assert_eq!(buf, check.as_slice());
        data.clear();
        let buf = data.as_slice();
        let nothing: &[u8] = &[];
        assert_eq!(nothing, buf);

        check.clear();

        fn push_both(bytes: &[u8], data: &mut DataBuffer, vec: &mut Vec<u8>) {
            data.push_bytes(bytes);
            vec.extend(bytes);
        }

        macro_rules! push_bytes {
            ($bytes:expr) => {
                {push_both($bytes, &mut data, &mut check);}
            };
        }
        push_bytes!(b"The quick brown fox jumps over the lazy dog.");
        push_bytes!(b"Some more bytes.");
        push_bytes!(b"Swallowing my bart because of how uncanny it is.");
        push_bytes!(b"I am so incredibly annoyed by how helix handles byte strings.");
        push_bytes!(b"I don't know how long the string is.");
        push_bytes!(b"I am so incredibly annoyed by how helix handles byte strings.");
        push_bytes!(b"Some more bytes.");
        push_bytes!(b"Swallowing my bart because of how uncanny it is.");
        assert_eq!(check.len(), data.len());
        assert!(data.capacity().is_power_of_two());
    }
}
