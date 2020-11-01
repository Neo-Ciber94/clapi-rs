#![allow(dead_code)]
mod args;
mod attr_data;
mod command;
mod option;

mod ext;
mod var;

pub(crate) use ext::*;

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::*;
use syn::*;
use syn::export::fmt::Display;
use crate::command::CommandAttribute;
use crate::var::{ArgLocalVar, LocalVarSource};

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as AttributeArgs);
    let func = syn::parse_macro_input!(item as ItemFn);

    let tokens = CommandAttribute::from_attribute_args(args, func).expand();
    println!("{}", tokens.to_string());

    tokens.into()

    // let t: TokenStream = quote! { x : &[String] }.into();
    // let vt = syn::parse_macro_input!(t as FnArg);
    // if let FnArg::Typed(pt) = vt {
    //     let var = ArgLocalVar::new(pt, LocalVarSource::Opts);
    //     println!("{}", var.expand().to_token_stream().to_string());
    // }
    //
    // let tokens = quote! {
    //     fn main(){
    //        println!("Hello World")
    //     }
    // };
    // tokens.into()
}

#[proc_macro_attribute]
pub fn subcommand(_: TokenStream, item: TokenStream) -> TokenStream { item }

#[proc_macro_attribute]
pub fn option(_: TokenStream, item: TokenStream) -> TokenStream { item }

#[proc_macro_attribute]
pub fn arg(_: TokenStream, item: TokenStream) -> TokenStream { item }

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