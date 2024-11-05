use std::process;
use std::{io, mem};

use libc::{tcgetattr, tcsetattr, termios as Termios, ECHO, ICANON, TCSAFLUSH};

// TODO: Rewrite with Mutexes and replace static mut.
static mut ORIGINAL_TERMIOS: Termios = unsafe { mem::zeroed() };

/*
  Enable Raw Mode for Terminal.
*/
pub fn enable_raw_mode() {
    let mut termios = get_terminal_attributes().unwrap_or_else(|err| {
        eprintln!("Could not fetch terminal attributes. Program will now exit. {err}");
        process::exit(1);
    });
    store_original_attributes(&termios);
    termios.c_lflag &= !(ECHO | ICANON);
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
        disable_raw_mode();
    }
}
