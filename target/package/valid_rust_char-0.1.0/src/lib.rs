//! A tiny crate that exports a table with all ranges of unicode scalar values of invalid
//! characters for a Rust file.
//!
//! This one it's created at build by checking validity with `TokenStream::from_str` with each
//! existing character so,althought this library it's kinda slow to build,the given list will be
//! the correct one for the version of Rust was built.

use std::char;
use std::iter::FusedIterator;
use std::ops::RangeInclusive;

include!("consts.rs");

fn bsearch_range_table<T: PartialOrd, R: core::ops::RangeBounds<T>>(
    c1: T,
    r: &[R],
) -> Result<usize, usize> {
    use core::cmp::Ordering::{Equal, Greater, Less};
    use core::ops::Bound::*;

    let c = &c1;

    r.binary_search_by(|r| match (r.start_bound(), r.end_bound()) {
        (Unbounded, Unbounded) => Equal,
        (Unbounded, Included(x)) => {
            if c <= x {
                Equal
            } else {
                Greater
            }
        }
        (Unbounded, Excluded(x)) => {
            if c < x {
                Equal
            } else {
                Greater
            }
        }
        (Included(x), Unbounded) => {
            if c >= x {
                Equal
            } else {
                Less
            }
        }
        (Excluded(x), Unbounded) => {
            if c > x {
                Equal
            } else {
                Less
            }
        }
        (Included(x), Included(y)) => {
            if c < x {
                Less
            } else if c > y {
                Greater
            } else {
                Equal
            }
        }
        (Excluded(x), Excluded(y)) => {
            if c <= x {
                Less
            } else if c >= y {
                Greater
            } else {
                Equal
            }
        }
        (Included(x), Excluded(y)) => {
            if c < x {
                Less
            } else if c >= y {
                Greater
            } else {
                Equal
            }
        }
        (Excluded(x), Included(y)) => {
            if c <= x {
                Less
            } else if c > y {
                Greater
            } else {
                Equal
            }
        }
    })
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

/// Checks whether a character it's an invalid one for a Rust file by binary searching [`RUST_INVALID_TABLE`].
#[inline]
pub fn is_invalid_rust_char(c: char) -> bool {
    bsearch_range_table(c as u32, &RUST_INVALID_TABLE).is_ok()
}

/// Checks whether a character it's a valid one for a Rust file,convenience to `!is_invalid_rust_char(c)`.
#[inline]
pub fn is_valid_rust_char(c: char) -> bool {
    !is_invalid_rust_char(c)
}

/// Creates an iterator that iterates thought all rust invalid characters,[`is_invalid_rust_char`]
/// should be preferred for search if one is there as otherwise the performance will decrease a lot.
#[inline]
pub fn all_invalid_rust_char() -> InvalidRustChars {
    InvalidRustChars {
        ranges: &RUST_INVALID_TABLE,
        all_chars: AllUnicode::new(),
    }
}

/// Iterator created with the function [`all_invalid_rust_char`].
pub struct InvalidRustChars {
    ranges: &'static [RangeInclusive<u32>],
    all_chars: AllUnicode,
}

impl Iterator for InvalidRustChars {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.ranges.first()?;
        let c1 = self.all_chars.next()?;
        let c = c1 as u32;

        if c < *r.start() || c > *r.end() {
            self.next()
        } else {
            if c == *r.end() {
                self.ranges = &self.ranges[1..];
            }

            Some(c1)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl DoubleEndedIterator for InvalidRustChars {
    fn next_back(&mut self) -> Option<Self::Item> {
        let r = self.ranges.last()?;
        let c1 = self.all_chars.next_back()?;
        let c = c1 as u32;

        if c < *r.start() || c > *r.end() {
            self.next_back()
        } else {
            if c == *r.start() {
                self.ranges = &self.ranges[..self.ranges.len() - 1];
            }

            Some(c1)
        }
    }
}

impl ExactSizeIterator for InvalidRustChars {
    fn len(&self) -> usize {
        use std::convert::TryInto;

        let r = match self.ranges.first() {
            Some(e) => e,
            None => return 0,
        };

        let diff = r.start()
            - if self.all_chars.0.is_empty() {
                if self.all_chars.1.is_empty() {
                    return 0;
                } else {
                    self.all_chars.1.start()
                }
            } else {
                self.all_chars.0.start()
            };

        self.ranges
            .iter()
            .fold(diff, |acc, e| acc + (e.end() - e.start()))
            .try_into()
            .expect("16-bit host failed to get the length as an usize.")
    }
}

impl FusedIterator for InvalidRustChars {}

/// Creates an iterator that iterates thought all rust valid characters,[`is_valid_rust_char`]
/// should be preferred for search if one is there as otherwise the performance will decrease a lot.
#[inline]
pub fn all_valid_rust_char() -> ValidRustChars {
    ValidRustChars {
        ranges: &RUST_INVALID_TABLE,
        all_chars: AllUnicode::new(),
    }
}

/// Iterator created with the function [`all_valid_rust_char`].
pub struct ValidRustChars {
    ranges: &'static [RangeInclusive<u32>],
    all_chars: AllUnicode,
}

impl Iterator for ValidRustChars {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let c1 = self.all_chars.next()?;
        let c = c1 as u32;
        let r = match self.ranges.first() {
            Some(e) => e,
            None => return Some(c1),
        };

        if c >= *r.start() && c < *r.end() {
            self.next()
        } else if c == *r.end() {
            self.ranges = &self.ranges[1..];
            self.next()
        } else {
            Some(c1)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl DoubleEndedIterator for ValidRustChars {
    fn next_back(&mut self) -> Option<Self::Item> {
        let c1 = self.all_chars.next_back()?;
        let c = c1 as u32;
        let r = match self.ranges.last() {
            Some(e) => e,
            None => return Some(c1),
        };

        if c > *r.start() && c <= *r.end() {
            self.next_back()
        } else if c == *r.start() {
            self.ranges = &self.ranges[..self.ranges.len() - 1];
            self.next_back()
        } else {
            Some(c1)
        }
    }
}

impl ExactSizeIterator for ValidRustChars {
    fn len(&self) -> usize {
        use std::convert::{TryFrom, TryInto};

        self.ranges
            .iter()
            .fold(
                u32::try_from(self.all_chars.len())
                    .expect("16-bit host failed to get the length as an usize."),
                |acc, e| acc - (e.end() - e.start()),
            )
            .try_into()
            .expect("16-bit host failed to get the length as an usize.")
    }
}

impl FusedIterator for ValidRustChars {}
