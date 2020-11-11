#![feature(proc_macro_span)]
#![allow(dead_code)]
extern crate proc_macro;

use crate::command::CommandData;
pub(crate) use ext::*;
use proc_macro::{TokenStream, Span};
use syn::{AttributeArgs, ItemFn};
use syn::export::ToTokens;

#[macro_use]
mod utils;
mod args;
mod command;
mod ext;
mod option;
mod var;
mod shared;

/// Marks and converts a function as a `Command`.
///
/// # Options:
/// - `description`: description of the command.
/// - `help`: help information about the command.
///
/// # Example:
/// ```text
/// #[command(description="", help=""]
/// fn main(){ }
/// ```
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as AttributeArgs);
    let func = syn::parse_macro_input!(item as ItemFn);

    let path = Span::call_site().source_file().path();
    let src = std::fs::read_to_string(path).unwrap();
    let file = syn::parse_file(&src).unwrap();

    CommandData::from_file(args, func, file)
        .expand()
        .into()
}

/// Marks a inner function as a subcommand.
///
/// # Options:
/// - `description`: description of the command.
/// - `help`: help information about the command.
///
/// # Example:
/// ```text
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
/// # Options
/// - `name` (required): name of the function argument.
/// - `description`: description of the option.
/// - `min`: min number of values the option takes.
/// - `max`: max number of values the option takes.
/// - `default`: default value(s) of the option.
///
/// # Example:
/// ```text
/// #[command]
/// #[option(name="x", description="", min=0, max=3, default=1,2,3)]
/// fn main(x: Vec<u32>){ }
/// ```
#[proc_macro_attribute]
pub fn option(_: TokenStream, _: TokenStream) -> TokenStream {
    // This just act as a marker
    panic!("option should be placed after a `command` or `subcommand` attribute")
}

/// Marks a function argument as command-line arguments.
///
/// # Options
/// - `name` (required): name of the function argument.
/// - `min`: min number of values the option takes.
/// - `max`: max number of values the option takes.
/// - `default`: default value(s) of the option.
///
/// # Example:
/// ```text
/// #[command]
/// #[arg(name="args", min=0, max=3, default="one", "two", "three")]
/// fn main(args: Vec<String>){ }
/// ```
#[proc_macro_attribute]
pub fn arg(_: TokenStream, _: TokenStream) -> TokenStream {
    // This just act as a marker
    panic!("arg should be placed after a `command` or `subcommand` attribute")
}