
use ::core::{
    ptr::NonNull,
};
use std::{
    alloc::{
        self,
        Layout,
    },
};

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
}
