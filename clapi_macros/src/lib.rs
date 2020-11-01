#![allow(dead_code)]
extern crate proc_macro;

use crate::command::CommandAttribute;
pub(crate) use ext::*;
use proc_macro::TokenStream;
use syn::{AttributeArgs, ItemFn};

mod args;
mod attr_data;
mod command;
mod ext;
mod option;
mod parser;
mod var;

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

    let tokens = CommandAttribute::from_attribute_args(args, func).expand();
    println!("{}", tokens.to_string());

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
pub fn subcommand(_: TokenStream, item: TokenStream) -> TokenStream {
    item
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
pub fn option(_: TokenStream, item: TokenStream) -> TokenStream {
    item
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
pub fn arg(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}
