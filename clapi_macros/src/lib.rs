#![feature(proc_macro_span)]
#![allow(dead_code)]
extern crate proc_macro;

use crate::command::CommandData;
pub(crate) use ext::*;
use proc_macro::{Span, TokenStream};
use std::path::PathBuf;
use syn::export::ToTokens;
use syn::{AttributeArgs, File, ItemFn};

#[macro_use]
mod utils;
mod args;
mod command;
mod ext;
mod keys;
mod option;
mod shared;
mod var;

/// Marks and converts a function as a command.
///
/// This is the entry point of a command line app, typically the marked function is `main`.
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
    let (path, file) = get_call_site_source_file();

    assertions::is_top_function(&func, &file);
    CommandData::from_file(args, func, path, file).expand().into()
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
/// fn main(){}
///
/// #[subcommand(description="", help=""]
/// fn test(){ }
/// ```
#[proc_macro_attribute]
pub fn subcommand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let (path, file) = get_call_site_source_file();

    shared::add_subcommand(shared::CommandRawData::new(
        attr.to_string(),
        item.to_string(),
        path
    ));

    let func = syn::parse_macro_input!(item as ItemFn);
    assertions::is_top_function(&func, &file);
    command::drop_command_attributes(func)
        .into_token_stream()
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

pub(crate) fn get_call_site_source_file() -> (PathBuf, File) {
    let path = Span::call_site().source_file().path();
    let src = std::fs::read_to_string(path.clone()).unwrap();
    (path, syn::parse_file(&src).unwrap())
}

mod assertions {
    use syn::{File, Item, ItemFn, Visibility};

    pub fn is_top_function(item_fn: &ItemFn, file: &File) {
        let found = file
            .items
            .iter()
            .filter_map(|item| matches_map!(item, Item::Fn(f) => f))
            // We don't compare attribute because we don't know the order they are expanded
            .any(|f| f.sig == item_fn.sig && f.vis == item_fn.vis && f.block == item_fn.block);

        if !found {
            panic!(
                "`{}` is not a top function.\
                \nCommand functions must be free functions and be declared outside a module.",
                item_fn.sig.ident
            )
        }
    }

    pub fn is_public(item_fn: &ItemFn) {
        match item_fn.vis {
            Visibility::Public(_) => {},
            _ => {
                panic!("subcommands must be declared public: `{}` is not public", item_fn.sig.ident);
            }
        }
    }
}
