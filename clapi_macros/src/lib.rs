// We check for nightly using `build.rs`
#![cfg_attr(nightly, feature(proc_macro_span))]
#![allow(dead_code)]
extern crate proc_macro;

use crate::command::CommandAttrData;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{AttributeArgs, ItemFn};

mod ext;
pub(crate) use ext::*;

#[macro_use]
mod utils;
mod arg;
mod command;
mod consts;
mod macro_attribute;
mod option;
mod query;
mod var;

/// Marks a function as a `command`.
///
/// This is the entry point of a command line app, typically the marked function is `main`.
///
/// # Options:
/// - `name`: Name of the command, by default is the `executable` name.
/// - `description`: Description of the command.
/// - `usage`: Information of the usage of the command.
/// - `help`: Help information about the command.
/// - `version`: Version of the command-line app.
///
/// # Example:
/// ```ignore
/// use clapi::macros::*;
///
/// #[command(description="A sample app", version=1.0)]
/// fn main(){
///     println!("Hello World!");
/// }
/// ```
#[cfg(not(nightly))]
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as AttributeArgs);
    let func = syn::parse_macro_input!(item as ItemFn);

    CommandAttrData::from_fn(args, func).expand().into()
}

/// Marks a function as a `command`.
///
/// This is the entry point of a command line app, typically the marked function is `main`.
///
/// # Options:
/// - `name`: Name of the command, by default is the `executable` name.
/// - `description`: Description of the command.
/// - `usage`: Information of the usage of the command.
/// - `help`: Help information about the command.
/// - `version`: Version of the command-line app.
///
/// # Example:
/// ```ignore
/// use clapi::macros::*;
///
/// #[command(description="A sample app", version=1.0)]
/// fn main(){
///     println!("Hello World!");
/// }
/// ```
#[cfg(nightly)]
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as AttributeArgs);
    let func = syn::parse_macro_input!(item as ItemFn);
    let path = call_site::path();

    CommandAttrData::from_path(args, func, path).expand().into()
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
/// - `name`: Name of the subcommand, by default is the function name.
/// - `description`: Description of the command.
/// - `usage`: Information of the usage of the command.
/// - `help`: Help information about the command.
/// - `version`: Version of the command-line app.
///
/// # Example:
/// ```ignore
/// use clapi::macros::*;
///
/// #[command]
/// fn main(){
///     #[subcommand(description="A test function")]
///     fn test(){
///         println!("This is a test");
///     }
/// }
/// ```
#[proc_macro_attribute]
#[allow(unreachable_code, unused_mut)]
pub fn subcommand(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_fn = syn::parse_macro_input!(item as ItemFn);

    #[cfg(not(nightly))]
    {
        // SAFETY: The `subcommand` attribute is removed by the root `command` when is an inner function.
        panic!("invalid function: `{}`\nfree function `subcommand`s are only supported in nightly builds", item_fn.sig.ident);
    }

    if !command::contains_expressions(&item_fn) {
        utils::insert_allow_dead_code_attribute(&mut item_fn);
    }

    // We need to drop all the `clapi` attributes to prevent `option` or `arg` panics
    command::drop_command_attributes(item_fn)
        .into_token_stream()
        .into()
}

// Change `require_assign` to?
// TODO: #[option(name, assignable=true)]
// TODO: #[option(name, assign=true)]

/// Declares a command option.
///
/// Any option declaration must contains the name of the argument for example:
/// `#[option(name)]`.
///
/// By default any function argument is considered a command `option`,
/// Use this attribute to provide additional information like `arg`, `alias`,
/// `description` or `min`, `max`, `default` and `values` arguments.
///
/// # Options
/// - `name`: Name of the option, by default is the function argument name.
/// - `arg`: Name of the option argument, by default is the function argument name.
/// - `alias`: Alias of the function argument.
/// - `description`: Description of the option.
/// - `min`: Min number of values the option takes.
/// - `max`: Max number of values the option takes.
/// - `default`: Default value(s) of the option.
/// - `values`: Valid values of the option.
/// - `hidden`: If the option is hidden for the help.
/// - `multiple`: If the option allow multiple declarations.
/// - `flag`: If the option is a bool flag, by default is `true`
/// - `error`: Error show when the value is invalid.
/// - `require_assign`: If the option requires to use `=` to assign the value, by default false,
///
/// Function arguments can be declared as the following types:
/// - Any type that implement `FromStr`.
/// - `Vec<T>` where `T` implements `FromStr`.
/// - `&[T]` slices where `T` implements `FromStr`.
/// - `Option<T>` where `T` implements `FromStr`.
///
/// # Example:
/// ```ignore
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
/// ```
#[proc_macro_attribute]
pub fn option(_: TokenStream, _: TokenStream) -> TokenStream {
    // This just act as a marker
    panic!("`option` should be placed after a `command` or `subcommand` attribute")
}

/// Declares a command argument.
///
/// Any argument declaration must contains the name of the argument for example:
/// `#[arg(name)]`.
///
/// # Options
/// - `name`: Name of the argument, by default is the function argument name.
/// - `min`: Min number of values the argument takes.
/// - `max`: Max number of values the argument takes.
/// - `default`: Default value(s) of the argument.
/// - `values`: Valid values of the argument.
/// - `error`: Error show when the value is invalid.
///
/// Function arguments can be declared as the following types:
/// - Any type that implement `FromStr`.
/// - `Vec<T>` where `T` implements `FromStr`.
/// - `&[T]` slices where `T` implements `FromStr`.
/// - `Option<T>` where `T` implements `FromStr`.
///
/// # Examples:
/// ```ignore
/// use clapi::macros::*;
///
/// #[command]
/// #[arg(args, min=1, max=10, default="Hello World")]
/// fn main(args: Vec<String>){
///     println!("{}", args.join(" "));
/// }
/// ```
#[proc_macro_attribute]
pub fn arg(_: TokenStream, _: TokenStream) -> TokenStream {
    // This just act as a marker
    panic!("`arg` should be placed after a `command` or `subcommand` attribute")
}

/// Specify the function that provides a help message for a command.
#[proc_macro_attribute]
#[allow(unused_variables, unreachable_code)]
pub fn command_help(_: TokenStream, item: TokenStream) -> TokenStream {
    #[cfg(not(nightly))]
    {
        panic!("`#[command_help]` is only available in nightly builds");
    }

    item
}

/// Specify the function that provides a usage message for a command.
#[proc_macro_attribute]
#[allow(unused_variables, unreachable_code)]
pub fn command_usage(_: TokenStream, item: TokenStream) -> TokenStream {
    #[cfg(not(nightly))]
    {
        panic!("`#[command_usage]` is only available in nightly builds");
    }

    item
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
