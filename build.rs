use std::fs::File;
use std::error::Error;
use std::char;
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>> {
    let mut it = invalid_rust_table().map(u32::from);
    let first = it.next().unwrap();
    let (mut lb, mut hb) = (first, first);

    let mut vec = Vec::with_capacity(1024);

    for c in it {
        if hb == (c - 1) {
            hb += 1;
        } else {
            vec.push(lb..=hb);
            lb = c;
            hb = c;            
        }
    }

    let mut f = File::create("src/consts.rs")?;

    writeln!(f, "/// A table with all ranges of unicode scalar values of characters that are not")?;
    writeln!(f, "/// valid in Rust.")?;
    writeln!(f, "pub const RUST_INVALID_TABLE: [core::ops::RangeInclusive<u32>; {}] = {:#?};", vec.len(), vec)?;
    Ok(())   
}

fn invalid_rust_table() -> impl Iterator<Item = char> {
    all_unicode().filter(is_not_rust_char)
}

fn all_unicode() -> impl Iterator<Item = char> {
    (0..=char::MAX as u32).filter_map(char::from_u32)
}

fn is_not_rust_char(c: &char) -> bool {
    let mut buf = [0; 4];

    match *c {
        '[' => false,
        ']' => false,
        '(' => false,
        ')' => false,
        '{' => false,
        '}' => false,
        '<' => false,
        '>' => false,
        '\'' => false,
        '"' => false,
        '_' => false,
        '\\' => false,
        _ => c.encode_utf8(&mut buf).parse::<proc_macro2::TokenStream>().is_err()
    }
}