//! # Clapi
//!
//! Clapi (Command-Line API) is a framework for create command line applications.
//!
//! Currently clapi provides several methods for create command line applications:
//! - Parsing the arguments
//! - Function handlers
//! - Macros attributes
//!
//! ## Parsing the arguments
//! ```no_run
//! // TODO
//! ```
//!
//! ## Function handlers
//! ```no_run
//! // TODO
//! ```
//!
//! ## Macro attributes
//! Requires `macros` feature enable.
//!
//! ```ignore
//! #[!cfg(features="macros")]
//! use clapi::macros::*;
//!
//! // We need to mark the app entry point as a `command`
//! #[command(version=1.0)]
//! fn main(){ }
//!
//! // Mark a function as a `subcommand` and defines if `option` and `arg`
//! #[subcommand]
//! #[option(times, alias="t", default=1)]
//! #[arg(values)]
//! fn repeat(times: u32, values: Vec<String>){
//!     let values = values.join(" ");
//!     for _ in 0..times {
//!         println!("{}", values);
//!     }
//! }
//! ```

/// Utilities and extensions intended for internal use.
#[macro_use]
pub mod utils;

#[cfg(feature = "serde")]
mod serde;

mod arg_count;
mod args;
mod command;
mod command_line;
mod context;
mod error;
mod option;
mod parse_result;
mod parser;
mod symbol;
mod tokenizer;

/// Utilities for provide suggestions.
pub mod suggestion;

/// Utilities for provide commands help information.
pub mod help;

// Re-exports
pub use self::arg_count::*;
pub use self::args::*;
pub use self::command::*;
pub use self::command_line::*;
pub use self::context::*;
pub use self::error::*;
pub use self::option::*;
pub use self::parse_result::*;
pub use self::parser::*;
pub use self::symbol::*;
pub use self::tokenizer::*;

#[cfg(feature = "macros")]
pub mod macros {
    extern crate clapi_macros;
    pub use clapi_macros::*;
}

#[cfg(feature = "macros")]
pub use macros::*;

#[macro_use]
mod app_macros;
pub use app_macros::*;

/// Utilities intended for internal use.
#[doc(hidden)]
pub mod internal {
    pub extern crate clapi_macros_internal;

    // These macros are used in `app_macros::app!` for declare the command option and args.
    // This was implemented with `proc_macro` to provide a type aware declaration of the variables,
    // currently in rust `Vec<$type:ty>` and `$type:ty` could be considered the same.
    //
    // In the `app_macros::app!` with declare the variables as:
    // `let $arg_name : $arg_type = $crate::declare_argument_var!(arguments, $arg_name: $arg_type);`
    //
    // We give a name to the variable outside the `proc_macro` to allow the IDE to provide type
    // information of the actual variable.
    // (This was only tested in intellij with the rust plugin version `0.3.137.3543-203`)

    #[doc(hidden)]
    #[macro_export]
    macro_rules! declare_option_var {
        ($options:ident, $name:ident: $ty:ty) => {
            $crate::internal::clapi_macros_internal::__declare_option_var!($options, $name: $ty)
        };
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! declare_argument_var {
        ($arguments:ident, $name:ident: $ty:ty) => {
            $crate::internal::clapi_macros_internal::__declare_argument_var!($arguments, $name: $ty)
        };
    }

    // todo: remove
    // #[doc(hidden)]
    // #[macro_export]
    // macro_rules! var_type {
    //     ($ty:ty) => {
    //         $crate::internal::clapi_macros_internal::__var_type!($ty)
    //     };
    // }
}

#[doc(hidden)]
pub use internal::*;
