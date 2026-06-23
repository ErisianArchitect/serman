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

// use std::time::Duration;

// use serman::{ForkContext};

// //= src/forker.rs::real_main
// fn real_main(ctx: ForkContext) -> serman::Result<()> {
//     let ctx2 = ctx.clone();
//     let pid = std::process::id();
//     match ctx.restart_id() {
//         0 => println!("First Run (pid: {pid})"),
//         count @ 1..=4 => {
//             println!("Restart Run {count} (pid: {pid})")
//         },
//         5 => {
//             println!("Final Restart Run (pid: {pid})");
//             return Ok(());
//         }
//         _ => return Ok(()),
//     }
//     ctx2.restart()?;
//     println!("Gonna exit.");
//     std::process::exit(0);
//     // Ok(())
// }

// //= src/forker.rs::main
// fn main() {
//     unsafe {
//         match serman::entry(real_main) {
//             serman::EntryResult::Parent(_) => println!("Parent exited."),
//             serman::EntryResult::Child(_) => println!("Child exited Normally."),
//         }
//     }
// }

fn main() {
    
}
