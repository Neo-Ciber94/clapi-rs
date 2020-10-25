#![allow(dead_code)]
mod args;
mod attr_data;
mod command;
mod option;

mod ext;
mod var;

pub(crate) use ext::*;

extern crate proc_macro;
use crate::attr_data::AttributeData;
use proc_macro::TokenStream;
use proc_macro::TokenTree;
use quote::*;
use syn::*;
use syn::export::fmt::Display;
use crate::option::OptionTokens;

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    // let args = syn::parse_macro_input!(attr as AttributeArgs);
    // let func = syn::parse_macro_input!(item as ItemFn);

    let mut opt = OptionTokens::new("test".to_string());
    opt.set_alias("t".to_string());
    opt.set_description("enable tests".to_string());

    let tokens = quote! {
        fn main(){
            let x = #opt;
            println!("{}", x.name());
            println!("{:?}", x.aliases());
            println!("{:?}", x.description());
            println!("{:#?}", x.args());
        }
    };

    tokens.into()
}

pub(crate) fn parse_with<T: syn::parse::Parser>(
    parser: T,
    stream: TokenStream,
) -> syn::Result<T::Output> {
    syn::parse::Parser::parse(parser, stream)
}

pub(crate) fn parse_to_stream<S: ToString>(s: S) -> TokenStream {
    use std::str::FromStr;
    TokenStream::from_str(&s.to_string()).unwrap()
}

pub(crate) fn parse_to_stream2<S: ToString>(s: S) -> proc_macro2::TokenStream {
    use std::str::FromStr;
    proc_macro2::TokenStream::from_str(&s.to_string()).unwrap()
}

pub(crate) fn parse_to_str_stream2<S: Display>(s: S) -> proc_macro2::TokenStream{
    let value = format!("\"{}\"", s.to_string());
    parse_to_stream2(value)
}
