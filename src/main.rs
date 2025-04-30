use ethereum_types::U256;
use hex::decode;
use std::io::{self, Write};

// ABI type definition

#[derive(Debug, Clone)]
enum AbiType {
    Address,
    Uint256,
    Bool,
    String,
    Bytes,
    Uint256Array,
    AddressArray,
}

impl AbiType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "address" => Some(Self::Address),
            "uint256" => Some(Self::Uint256),
            "bool" => Some(Self::Bool),
            "string" => Some(Self::String),
            "bytes" => Some(Self::Bytes),
            "uint256[]" => Some(Self::Uint256Array),
            "address[]" => Some(Self::AddressArray),
            _ => None,
        }
    }

    fn is_dynamic(&self) -> bool {
        matches!(
            self,
            Self::String | Self::Bytes | Self::Uint256Array | Self::AddressArray
        )
    }
}

// helper decoders
fn decode_selector(data: &[u8]) -> String {
    format!("0x{}", hex::encode(&data[..4]))
}

fn decode_address(chunk: &[u8]) -> String {
    format!("0x{}", hex::encode(&chunk[12..]))
}

fn decode_uint256(chunk: &[u8]) -> String {
    U256::from_big_endian(chunk).to_string()
}

fn decode_bool(chunk: &[u8]) -> String {
    if chunk[31] == 1 { "true" } else { "false" }.to_string()
}

fn as_usize(chunk: &[u8]) -> usize {
    U256::from_big_endian(chunk).as_usize()
}

fn padded(n: usize) -> usize {
    (n + 31) & !31
}

fn decode_calldata(param_types: &[AbiType], data: &[u8]) {
    println!("Function selector: {}", decode_selector(data));

    let head = &data[4..]; // skip selector
    for (idx, ty) in param_types.iter().enumerate() {
        let slot = &head[idx * 32..(idx + 1) * 32];
        println!("\nParam {idx} ({ty:?})");

        if !ty.is_dynamic() {
            match ty {
                AbiType::Address => println!("  address   = {}", decode_address(slot)),
                AbiType::Uint256 => println!("  uint256   = {}", decode_uint256(slot)),
                AbiType::Bool => println!("  bool      = {}", decode_bool(slot)),
                _ => unreachable!(),
            }
        } else {
            // for dynamic
            let offset = as_usize(slot);
            let dyn_start = 4 + offset; // 4 byte selector proceeds the head
            let len_chunk = &data[dyn_start..dyn_start + 32];
            let len = as_usize(len_chunk);
            let payload_start = dyn_start + 32;
            let payload_end = payload_start + padded(len);
            let payload = &data[payload_start..payload_start + len];

            match ty {
                AbiType::String => match std::str::from_utf8(payload) {
                    Ok(s) => println!("  string    = \"{s}\""),
                    Err(_) => println!("  string    = (invalid UTF-8) 0x{}", hex::encode(payload)),
                },
                AbiType::Bytes => println!("  bytes     = 0x{}", hex::encode(payload)),
                AbiType::Uint256Array => {
                    println!("  uint256[] length = {len}");
                    for i in 0..len {
                        let el = &payload[i * 32..(i + 1) * 32];
                        println!("    [{i}] = {}", decode_uint256(el));
                    }
                }
                AbiType::AddressArray => {
                    println!("  address[] length = {len}");
                    for i in 0..len {
                        let el = &payload[i * 32..(i + 1) * 32];
                        println!("    [{i}] = {}", decode_address(el));
                    }
                }
                _ => {}
            }

            println!(
                "  (dynamic section bytes {} – {})",
                dyn_start,
                payload_end - 1
            );
        }
    }
}

fn ask(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).expect("read failure");
    buf.trim().to_string()
}

fn main() {
    // 1) parameter type list
    let type_line = ask("Enter parameter types (comma-separated, e.g. address,uint256,string): ");
    let param_types: Vec<AbiType> = type_line
        .split(',')
        .filter_map(|s| AbiType::from_str(s.trim()))
        .collect();

    if param_types.is_empty() {
        eprintln!("Could not parse any types – aborting.");
        return;
    }

    // 2) calldata hex
    let hex_input = ask("Enter calldata hex (with or without 0x): ");
    let bytes = match decode(hex_input.trim_start_matches("0x")) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Invalid hex: {e}");
            return;
        }
    };

    // 3) decode
    decode_calldata(&param_types, &bytes);
}
