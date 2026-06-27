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
//:---[END-HEADER]---

#[cfg(not(unix))]
compile_error!("This library is for unix systems, and the target is not a unix system.");

#[cfg(unix)] mod fd;
#[cfg(unix)] mod fd_flags;
#[cfg(unix)] mod ref_count;
#[cfg(unix)] mod util;

#[cfg(unix)] pub mod context;
#[cfg(unix)] pub mod data;
#[cfg(unix)] pub mod entry;
#[cfg(unix)] pub mod error;
#[cfg(unix)] pub mod ffi;
#[cfg(unix)] pub mod io;
#[cfg(unix)] pub mod messaging;
