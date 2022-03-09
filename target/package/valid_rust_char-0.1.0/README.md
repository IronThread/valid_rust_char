# valid_rust_char

A tiny crate that exports a table with all ranges of unicode scalar values of invalid
characters for a Rust file.
 
This one it's created at build by checking validity with `TokenStream::from_str` with each 
existing character so,althought this library it's kinda slow to build,the given list will be
the correct one for the version of Rust was built.
