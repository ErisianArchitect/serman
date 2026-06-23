
use serman::{
    ffi::{
        pipe,
        
    },
    io::{
        Reader,
        Writer,
    },
    error::{
        // TODO: I probably don't need all of this, but I wanted to include it just in case so that I wouldn't have to come back here.
        Error, Result,
        PipeError, PipeResult,
        FdError, FdResult,
        ReadError, ReadResult,
        WriteError, WriteResult,
    }
};

// For making RGB FG ansi wrapped text.
macro_rules! ansic {
    (($r:literal, $g:literal, $b: literal), $text:literal) => {
        concat!(
            "\x1b[38;2;",
            stringify!($r),
            ";",
            stringify!($g),
            ";",
            stringify!($b),
            "m",
            $text,
            "\x1b[39m",
        )
    };
}

const PASS_TEXT: &'static str = ansic!((000, 255, 100), "-----------\nTEST PASSED\n-----------");
const FAIL_TEXT: &'static str = ansic!((255, 050, 050), "-----------\nTEST FAILED\n-----------");

fn print_pass(pass: bool) {
    if pass {
        println!("{PASS_TEXT}");
    } else {
        println!("{FAIL_TEXT}");
    }
}

#[test]
pub fn pipe_read_write_test() -> Result<()> {
    let fds = unsafe { pipe()? };
    let mut reader = Reader::from_fd(fds.reader)?;
    let mut writer = Writer::from_fd(fds.writer)?;

    let orig = "test";
    
    assert_eq!(orig.len(), writer.write_all(orig.as_bytes())?);

    let mut buf = [0u8; 4];
    assert_eq!(buf.len(), reader.read_exact(&mut buf)?);

    let s = unsafe { str::from_utf8_unchecked(&buf) };

    assert_eq!(s, orig);

    print_pass(true);
    
    Ok(())
}
