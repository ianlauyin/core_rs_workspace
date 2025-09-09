use std::fmt::Write as _;

use bytes::Bytes;

pub fn cstr_literal(bytes: &[u8]) -> String {
    let mut repr = String::new();
    for chunk in bytes.utf8_chunks() {
        write!(repr, "{}", chunk.valid()).unwrap();

        for byte in chunk.invalid() {
            write!(repr, "\\x{:02X}", byte).unwrap();
        }
    }
    repr
}

fn main() {
    let mem = Bytes::from("hello world中文");
    let lit = cstr_literal(&mem);
    println!("{lit}")
}
