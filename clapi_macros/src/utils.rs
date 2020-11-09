use std::fmt::Display;
use std::str::FromStr;
use proc_macro2::{TokenStream, LexError};

pub fn to_stream2<S: Display>(s: S) -> Result<TokenStream, LexError> {
    TokenStream::from_str(&s.to_string())
}

pub fn to_str_literal_stream2<S: Display>(s: S) -> Result<TokenStream, LexError> {
    let value = format!("\"{}\"", s.to_string());
    to_stream2(value)
}