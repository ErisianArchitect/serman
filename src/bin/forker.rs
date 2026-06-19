

use std::time::Duration;

use serman::{ForkContext};

fn real_main(ctx: ForkContext) -> serman::Result<()> {
    let ctx2 = ctx.clone();
    let pid = std::process::id();
    match ctx.restart_id() {
        0 => println!("First Run (pid: {pid})"),
        count @ 1..=4 => {
            println!("Restart Run {count} (pid: {pid})")
        },
        5 => {
            println!("Final Restart Run (pid: {pid})");
            return Ok(());
        }
        _ => return Ok(()),
    }
    ctx2.restart()?;
    println!("Gonna exit.");
    std::process::exit(0);
    // Ok(())
}

fn main() {
    unsafe {
        match serman::entry(real_main) {
            serman::EntryResult::Parent(_) => println!("Parent exited."),
            serman::EntryResult::Child(_) => println!("Child exited Normally."),
        }
    }
}
