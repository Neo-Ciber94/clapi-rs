// We check for nightly using `build.rs`
#![cfg_attr(nightly, feature(proc_macro_span))]

#![allow(dead_code)]
extern crate proc_macro;

use crate::command::CommandData;
pub(crate) use ext::*;
use proc_macro::TokenStream;
use syn::export::ToTokens;
use syn::{AttributeArgs, ItemFn};

#[macro_use]
mod utils;
mod args;
mod assertions;
mod attr;
mod command;
mod ext;
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
#[cfg(not(nightly))]
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as AttributeArgs);
    let func = syn::parse_macro_input!(item as ItemFn);

    CommandData::from_fn(args, func)
        .expand()
        .into()
}

#[cfg(nightly)]
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as AttributeArgs);
    let func = syn::parse_macro_input!(item as ItemFn);

    let path = call_site::path();
    CommandData::from_path(args, func, path)
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
/// fn main(){}
///
/// #[subcommand(description="", help=""]
/// fn test(){ }
/// ```
#[proc_macro_attribute]
#[allow(unreachable_code)]
pub fn subcommand(_: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as ItemFn);

    #[cfg(not(nightly))]
    {
        // SAFETY: The `subcommand` attribute is removed by the root `command` when is a inner function.
        panic!("invalid function: `{}`\nfree function `subcommand`s are only supported in nightly builds", func.sig.ident);
    }

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

#[cfg(nightly)]
mod call_site {
    use proc_macro::Span;
    use std::path::PathBuf;
    use syn::File;

    pub fn path() -> PathBuf {
        Span::call_site().source_file().path()
    }

    pub fn source_file() -> (PathBuf, File) {
        let path = path();
        let src = std::fs::read_to_string(&path).unwrap();
        (path, syn::parse_file(&src).unwrap())
    }
}
