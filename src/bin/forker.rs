

use std::time::Duration;

use serman::{Context};

fn main() {
    fn main(mut ctx: Context) -> serman::Result<()> {
        let pid = std::process::id();
        match ctx.restart_count {
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
        ctx.restart()?;
        Ok(())
    }
    unsafe {
        match serman::entry(main) {
            serman::EntryResult::Parent(_) => println!("Parent exited."),
            serman::EntryResult::Child(_) => println!("Child exited."),
        }
    }
}
