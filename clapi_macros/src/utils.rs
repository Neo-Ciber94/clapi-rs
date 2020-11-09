use std::fmt::Display;
use std::str::FromStr;
use proc_macro2::{TokenStream, LexError};
use syn::PatType;
use syn::export::ToTokens;

pub fn to_stream2<S: Display>(s: S) -> Result<TokenStream, LexError> {
    TokenStream::from_str(&s.to_string())
}

pub fn to_str_literal_stream2<S: Display>(s: S) -> Result<TokenStream, LexError> {
    let value = format!("\"{}\"", s.to_string());
    to_stream2(value)
}

pub fn pat_type_to_string(pat_type: &PatType) -> String {
    let arg_name = pat_type.pat.to_token_stream().to_string();
    let type_name = pat_type.ty.to_token_stream().into_iter()
        .map(|t| t.to_string())
        .collect::<Vec<String>>()
        .join("");

    format!("{} : {}", arg_name, type_name)
}