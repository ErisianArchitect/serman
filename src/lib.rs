
#[cfg(not(unix))]
compile_error!("This library is for unix systems, and the target is not a unix system.");
