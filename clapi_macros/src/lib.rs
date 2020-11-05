#![allow(dead_code)]
extern crate proc_macro;

use crate::command::CommandAttribute;
pub(crate) use ext::*;
use proc_macro::TokenStream;
use syn::{AttributeArgs, ItemFn, Attribute};

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

    {
        use macro_attribute_args::*;
        use quote::*;

        fn parse_with<T: syn::parse::Parser>(parser: T, tokens: TokenStream) -> syn::Result<T::Output>{
            parser.parse(tokens)
        }

        let tokens = quote! {
            #[std::p::person(name="jhon", age=20, working=true, child(name="bob"))]
        };

        let raw_attr = parse_with(Attribute::parse_outer, tokens.into()).unwrap().remove(0);
        let attr = MacroAttributeArgs::new(raw_attr);

        println!("{:#?}", attr);
    }

    let tokens = CommandAttribute::from_attribute_args(args, func).expand();
    // println!("{}", tokens.to_string());
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
    // This just acts as a marker, the actual operation is performed by `command`
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
    // This just acts as a marker, the actual operation is performed by `command`
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
    // This just acts as a marker, the actual operation is performed by `command`
    item
}