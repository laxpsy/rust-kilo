use rust_kilo::RawModeGuard;
use std::io::{self, stdin, Read};

fn main() {
    let _raw_mode = RawModeGuard::new();
    println!("Raw mode enabled. Press any key...");

    for byte in io::stdin().bytes() {
        let byte = byte.unwrap();
        let character = byte as char;
        if character.is_control() {
            println!("Binary: {0:08b} ASCII: {0:#03} \r", byte);
        } else {
            println!(
                "Binary: {0:08b} ASCII: {0:#03} Character: {1:#?}\r",
                byte, character
            );
        }

        if character == 'q' {
            drop(_raw_mode);
            break;
        }
    }
}
