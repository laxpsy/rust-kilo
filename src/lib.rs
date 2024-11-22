use std::error::{self, Error};
use std::io::{Read, Write};
use std::process;
use std::{io, mem};

use libc::{
    tcgetattr, tcsetattr, termios as Termios, write, ECHO, ICANON, ICRNL, ISIG, IXON, OPOST,
    TCSAFLUSH, VMIN, VTIME,
};

// TODO: Rewrite with Mutexes and replace static mut.
static mut ORIGINAL_TERMIOS: Termios = unsafe { mem::zeroed() };

/*
  Enable Raw Mode for Terminal.
  ICRNL: disable translation of \r to \n.
  IXON: disable XON, XOFF
  ICANON: disable canonical mode
  ECHO: disable echoing of STDDIN
  ISIG: disable signal interrupts
  OPOST: disable output post-processing
        i.e conversion of /n to /r/n.
  VMIN: how many bytes in input buffer?
  VTIME: timeout in tenth-of-seconds
*/
pub fn enable_raw_mode() {
    let mut termios = get_terminal_attributes().unwrap_or_else(|err| {
        eprintln!("Could not fetch terminal attributes. Program will now exit. {err}");
        process::exit(1);
    });
    store_original_attributes(&termios);
    termios.c_iflag &= !(ICRNL | IXON);
    termios.c_oflag &= !(OPOST);
    termios.c_lflag &= !(ECHO | ICANON | ISIG);
    termios.c_cc[VMIN] = 1;
    termios.c_cc[VTIME] = 0;
    set_terminal_attributes(&termios).unwrap_or_else(|err| {
        eprintln!("Could not set terminal attributes. Program will now exit. {err}");
        process::exit(1);
    });
}

/*
  Restore original terminal attributes.
*/
pub fn disable_raw_mode() -> io::Result<()> {
    unsafe {
        wrap_with_result(tcsetattr(0, TCSAFLUSH, &ORIGINAL_TERMIOS))?;
        Ok(())
    }
}

/*
  Store the original terminal attributes
  to restore at the end of the program.
*/
fn store_original_attributes(termios: &Termios) {
    unsafe { ORIGINAL_TERMIOS = termios.clone() };
}

/*
  0 denotes STDIN_FILENO
*/
fn set_terminal_attributes(termios: &Termios) -> io::Result<()> {
    unsafe {
        wrap_with_result(tcsetattr(0, TCSAFLUSH, termios))?;
        Ok(())
    }
}

/*
  0 denotes STDIN_FILENO
*/
fn get_terminal_attributes() -> io::Result<Termios> {
    unsafe {
        let mut termios = mem::zeroed();
        wrap_with_result(tcgetattr(0, &mut termios))?;
        Ok(termios)
    }
}

/*
  Implement a wrap for error handling incase
  unsafe functions fail.
*/
fn wrap_with_result(result: i32) -> io::Result<()> {
    if result == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn editor_read_key() -> [u8; 1] {
    let mut buffer = [0; 1];
    io::stdin().read_exact(&mut buffer).unwrap_or_else(|err| {
        eprintln!("Failure in reading input from the user. Program will now exit! {err}");
        process::exit(1);
    });
    buffer
}

/*
    Handle quitting with Ctrl+Q.
    Note to self - add BITMASK logic
    for control characters.
    Instead of taking in RawModeGuard
    as a parameter, handle the case with
    Errors.
*/
pub fn editor_process_key() -> Result<(), ()> {
    const CTRL_Q: u8 = 0b00010001;
    let character_u8 = editor_read_key()[0];
    if character_u8 <= 31 || character_u8 == 127 {
        print!("{character_u8}\r\n");
    } else {
        print!("{0}, {1}\r\n", character_u8, character_u8 as char);
    }
    io::stdout().flush().unwrap();
    if character_u8 == CTRL_Q {
        return Err(());
    }
    Ok(())
}

/*
    Grab a lock to STDOUT stream,
    write byte \x1b[2J to clear screen
    and \x1b[H to reset cursor to
    original position.
    VT100 terminal commands.
*/

pub fn editor_refresh_screen() -> () {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(b"\x1b[2J\x1b[H").unwrap();
    handle.flush().unwrap();
}

/*
  Implementing RAII pattern to ensure Raw Mode
  is disabled when the program exits.
*/
pub struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> Self {
        enable_raw_mode();
        RawModeGuard
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        disable_raw_mode().unwrap_or_else(|err| {
            eprintln!("ERROR! Could not restore Terminal Attributes. {err}");
            process::exit(1);
        });
    }
}
