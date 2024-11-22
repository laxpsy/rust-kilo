use rust_kilo::{editor_process_key, editor_refresh_screen, RawModeGuard};
use std::io::{self, Read};

fn main() {
    let _raw_mode = RawModeGuard::new();
    print!("Raw mode enabled. Press any key...\r\n");

    loop {
        //  editor_refresh_screen();
        match editor_process_key() {
            Ok(()) => continue,
            Err(()) => {
                editor_refresh_screen();
                break;
            }
        }
    }
}
