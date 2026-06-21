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



use ::core::sync::atomic::{AtomicU32, Ordering};

#[repr(transparent)]
#[derive(Debug)]
pub struct RefCounter32 {
    count: AtomicU32,
}

impl RefCounter32 {
    #[must_use]
    #[inline(always)]
    pub fn new(count: u32) -> Self {
        Self {
            count: AtomicU32::new(count),
        }
    }

    #[must_use]
    #[inline(always)]
    pub fn count(&self) -> u32 {
        self.count.load(Ordering::Acquire)
    }

    /// Increments the count and returns the previous count.
    #[inline(always)]
    pub fn increment(&self) -> u32 {
        self.count.fetch_add(1, Ordering::SeqCst)
    }

    /// Decrements and returns `Ok(new_count)`.
    /// Returns `Err(())` if count was already 0.
    pub fn decrement(&self) -> Result<u32, ()> {
        // cas loop for decrementing so that we do not decrement beyond 0.
        let mut count = self.count.load(Ordering::Relaxed);
        if count == 0 {
            return Err(());
        }
        loop {
            let decremented_count = count - 1;
            match self.count.compare_exchange(
                count,
                decremented_count,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(decremented_count),
                Err(0) => return Err(()),
                // Although this is called current_count, it may change by the next time compare_exchange is called.
                Err(current_count) => count = current_count,
            }
            std::hint::spin_loop();
        }
    }
}
