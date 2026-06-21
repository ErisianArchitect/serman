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


#[macro_export]
macro_rules! select {
    ($cond: expr, $f:expr, $t:expr) => {
        if ($cond) {
            $t
        } else {
            $f
        }
    };
}

/// Branchless select.
#[must_use]
#[inline(always)]
pub const fn select_copy<T: Copy>(false_: T, true_: T, condition: bool) -> T {
    if condition {
        true_
    } else {
        false_
    }
}
