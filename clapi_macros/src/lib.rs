// We check for nightly using `build.rs`
#![cfg_attr(nightly, feature(proc_macro_span))]
#![allow(dead_code)]
extern crate proc_macro;

use crate::command::CommandData;
pub(crate) use ext::*;

use proc_macro::TokenStream;
use syn::export::ToTokens;
use syn::{AttributeArgs, ItemFn};

mod macro_attribute;

#[macro_use]
mod utils;
mod args;
mod assertions;
mod attr;
mod command;
mod ext;
mod option;
mod var;

/// Marks a function as a `command`.
///
/// This is the entry point of a command line app, typically the marked function is `main`.
///
/// # Options:
/// - `description`: Description of the command.
/// - `about`: Information about the command.
/// - `version`: Version of the command-line app.
///
/// # Example:
/// ```ignore no_run
/// use clapi::macros::*;
///
/// #[command(description="A sample app", version=1.0)]
/// fn main(){
///     println!("Hello World!");
/// }
///
/// // > cargo run
/// ```
#[cfg(not(nightly))]
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as AttributeArgs);
    let func = syn::parse_macro_input!(item as ItemFn);

    CommandData::from_fn(args, func).expand().into()
}

/// Marks a function as a `command`.
///
/// This is the entry point of a command line app, typically the marked function is `main`.
///
/// # Options:
/// - `description`: Description of the command.
/// - `about`: Information about the command.
/// - `version`: Version of the command-line app.
///
/// # Example:
/// ```text
/// use clapi::macros::*;
///
/// #[command(description="A sample app", version=1.0)]
/// fn main(){
///     println!("Hello World!");
/// }
///
/// // > cargo run
/// ```
#[cfg(nightly)]
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as AttributeArgs);
    let func = syn::parse_macro_input!(item as ItemFn);

    let path = call_site::path();
    CommandData::from_path(args, func, path).expand().into()
}

/// Marks a function as a `subcommand`.
///
/// ## Stable
/// Only inner functions of a `command` or `subcommand` can be declared as a subcommand.
///
/// ## Nightly
/// When compiling for `nightly` rust any free function or inner can be marked as a `subcommand`.
///
/// # Options:
/// - `description`: Description of the command.
/// - `help`: Help information about the command.
/// - `version`: Version of the subcommand.
///
/// # Example:
/// ```text
/// use clapi::macros::*;
///
/// #[command]
/// fn main(){
///     #[subcommand(description="A test function")]
///     fn test(){
///         println!("This is a test");
///     }
/// }
///
/// // > cargo run -- test
/// ```
#[proc_macro_attribute]
#[allow(unreachable_code)]
pub fn subcommand(_: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as ItemFn);

    #[cfg(not(nightly))]
    {
        // SAFETY: The `subcommand` attribute is removed by the root `command` when is an inner function.
        panic!("invalid function: `{}`\nfree function `subcommand`s are only supported in nightly builds", func.sig.ident);
    }

    command::drop_command_attributes(func)
        .into_token_stream()
        .into()
}

/// Declares a command option.
///
/// Any option declaration must contains the name of the argument for example:
/// `#[option(name)]`.
///
/// By default any function argument is considered a command `option`,
/// Use this attribute to provide additional information like `arg`, `alias`,
/// `description` or `min`, `max` and `default` arguments.
///
/// # Options
/// - `arg`: Name of the argument.
/// - `alias`: Alias of the function argument.
/// - `description`: Description of the option.
/// - `min`: Min number of values the option takes.
/// - `max`: Max number of values the option takes.
/// - `default`: Default value(s) of the option.
///
/// Function arguments can be declared as the following types:
/// - Any type that implement `FromStr`.
/// - `Vec<T>` where `T` implements `FromStr`.
/// - `&[T]` slices where `T` implements `FromStr`.
/// - `Option<T>` where `T` implements `FromStr`.
///
/// # Example:
/// ```text
/// use clapi::macros::*;
///
/// #[command]
/// #[option(repeat, alias="r", default=1)]
/// #[option(upper_case, alias="u", description="Display the message in uppercase")]
/// fn main(repeat: u32, upper_case: bool){
///     for _ in 0..repeat {
///         if upper_case {
///             println!("HELLO WORLD");
///         } else {
///             println!("hello world");
///         }
///     }
/// }
///
/// // > cargo run -- --repeat -u
/// ```
#[proc_macro_attribute]
pub fn option(_: TokenStream, _: TokenStream) -> TokenStream {
    // This just act as a marker
    panic!("option should be placed after a `command` or `subcommand` attribute")
}

/// Declares a command argument.
///
/// Any argument declaration must contains the name of the argument for example:
/// `#[arg(name)]`.
///
/// # Options
/// - `arg`: Name of the argument.
/// - `min`: Min number of values the option takes.
/// - `max`: Max number of values the option takes.
/// - `default`: Default value(s) of the option.
///
/// Function arguments can be declared as the following types:
/// - Any type that implement `FromStr`.
/// - `Vec<T>` where `T` implements `FromStr`.
/// - `&[T]` slices where `T` implements `FromStr`.
/// - `Option<T>` where `T` implements `FromStr`.
///
/// # Examples:
/// ```text
/// use clapi::macros::*;
///
/// #[command]
/// #[arg(name, min=1, max=10, default="Hello World")]
/// fn main(args: Vec<String>){
///     println!("{}", args.join(" "));
/// }
///
/// // > cargo run -- one two three
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
