use ethereum_types::U256;
use hex::{decode, encode};
use std::io::{self, Write};

fn take_input() -> String {
    print!("Enter calldata (hex, with or without 0x): ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).expect("stdin read failed");
    buf.trim().trim_start_matches("0x").to_string()
}

fn decode_selector(data: &[u8]) -> String {
    format!("0x{}", encode(&data[0..4]))
}

fn decode_address(chunk: &[u8]) -> String {
    format!("0x{}", encode(&chunk[12..])) // last 20 bytes
}

fn decode_uint256(chunk: &[u8]) -> String {
    let n = U256::from_big_endian(chunk);
    n.to_string()
}

fn main() {
    let hex_input = take_input();
    let bytes = decode(&hex_input).expect("Invalid input");

    println!("\nFunction selector: {}", decode_selector(&bytes));

    for (i, chunk) in bytes[4..].chunks(32).enumerate() {
        if chunk.len() != 32 {
            println!("Param {i}: <incomplete 32-byte slot>");
            continue;
        }
        println!("\nParam {i}  (0x{}):", encode(chunk));

        if chunk[..12] == [0u8; 12] {
            println!("  address  = {}", decode_address(chunk));
        }
        println!("  uint256  = {}", decode_uint256(chunk));
    }
}
