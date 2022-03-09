use std::char;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::iter::FusedIterator;
use std::ops::RangeInclusive;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    if !Path::new("src/consts.rs").exists() {
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

        writeln!(
            f,
            "/// A table with all ranges of unicode scalar values of characters that are not"
        )?;
        writeln!(f, "/// valid in Rust.")?;
        writeln!(
            f,
            "pub const RUST_INVALID_TABLE: [core::ops::RangeInclusive<u32>; {}] = {:#?};",
            vec.len(),
            vec
        )?;
    }
    Ok(())
}

fn invalid_rust_table() -> impl Iterator<Item = char> {
    AllUnicode::new().filter(is_not_rust_char)
}

struct AllUnicode(RangeInclusive<u32>, RangeInclusive<u32>);

impl AllUnicode {
    #[inline]
    pub fn new() -> Self {
        Self(0..=0xD800 - 1, 0xDFFF + 1..=char::MAX as u32)
    }
}

impl Default for AllUnicode {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for AllUnicode {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        self.0
            .next()
            .or_else(|| self.1.next())
            .and_then(char::from_u32)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl DoubleEndedIterator for AllUnicode {
    #[inline]
    fn next_back(&mut self) -> Option<char> {
        self.1
            .next_back()
            .or_else(|| self.0.next_back())
            .and_then(char::from_u32)
    }
}

impl ExactSizeIterator for AllUnicode {
    #[inline]
    fn len(&self) -> usize {
        self.0.size_hint().0 + self.1.size_hint().0
    }
}

impl FusedIterator for AllUnicode {}

fn is_not_rust_char(c: &char) -> bool {
    let mut buf = [0; 8];

    match *c {
        '{' => false,
        '[' => false,
        '(' => false,
        '<' => false,
        '}' => false,
        ']' => false,
        ')' => false,
        '>' => false,
        c => unsafe {
            write!(&mut buf[..], "{0}{0}", c);

            std::str::from_utf8_unchecked(&buf[..c.len_utf8() * 2]).parse::<proc_macro2::TokenStream>()
            .is_err()
        }
    }
}
