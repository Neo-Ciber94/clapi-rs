use std::fmt::Display;
use std::str::FromStr;
use proc_macro2::{TokenStream, LexError};

pub fn parse_to_stream2<S: Display>(s: S) -> Result<TokenStream, LexError> {
    TokenStream::from_str(&s.to_string())
}

pub fn parse_to_str_stream2<S: Display>(s: S) -> Result<TokenStream, LexError> {
    let value = format!("\"{}\"", s.to_string());
    parse_to_stream2(value)
}