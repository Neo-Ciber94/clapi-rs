#![feature(proc_macro_span)]
//#![allow(dead_code)]
extern crate proc_macro;

use crate::command::CommandFromFn;
pub(crate) use ext::*;
use proc_macro::{TokenStream, Span};
use syn::{AttributeArgs, ItemFn};
use syn::export::ToTokens;

mod args;
mod command;
mod ext;
mod option;
mod utils;
mod var;
mod shared;

/// Marks and converts a function as a `Command`.
///
/// # Example:
/// ``` ignore
/// #[command(description="", help=""]
/// fn main(){ }
/// ```
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as AttributeArgs);
    let func = syn::parse_macro_input!(item as ItemFn);

    // let tokens = if cfg!(target_feature="proc_macro_span") {
    //     let path = Span::call_site().source_file().path();
    //     let src = std::fs::read_to_string(path).unwrap();
    //     let file = syn::parse_file(&src).unwrap();
    //     CommandFromFn::from_file(args, func, file).expand()
    // } else {
    //     CommandFromFn::from_fn(args, func).expand()
    // };

    let path = Span::call_site().source_file().path();
    let src = std::fs::read_to_string(path).unwrap();
    let file = syn::parse_file(&src).unwrap();
    let tokens = CommandFromFn::from_file(args, func, file).expand();

    //println!("{}", tokens.to_string());
    tokens.into()
}

/// Marks a inner function as a subcommand.
///
/// # Example:
/// ```ignore
/// #[command]
/// fn main(){
///     #[subcommand(description="", help=""]
///     fn test(){ }
/// }
/// ```
#[proc_macro_attribute]
pub fn subcommand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let raw_attr_args = attr.to_string();
    let raw_item_fn = item.to_string();
    let func = syn::parse_macro_input!(item as ItemFn);

    shared::get_subcommand_registry().push(shared::CommandRawData::new(raw_attr_args, raw_item_fn));

    command::drop_command_attributes(func)
        .to_token_stream()
        .into()
}

/// Adds command-line option information to a function argument.
///
/// # Example:
/// ```ignore
///
/// #[command]
/// #[option(name="x", description="", min=0, max=3, default=1,2,3)]
/// fn main(x: Vec<u32>){ }
/// ```
#[proc_macro_attribute]
pub fn option(_: TokenStream, _: TokenStream) -> TokenStream {
    panic!("option should be placed after a `command` or `subcommand` attribute")
}

/// Marks a function argument as command-line arguments.
///
/// # Example:
/// ```ignore
///
/// #[command]
/// #[arg(name="args", min=0, max=3, default="one", "two", "three")]
/// fn main(args: Vec<String>){ }
/// ```
#[proc_macro_attribute]
pub fn arg(_: TokenStream, _: TokenStream) -> TokenStream {
    panic!("arg should be placed after a `command` or `subcommand` attribute")
}