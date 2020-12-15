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

#[cfg(feature="serde")]
pub mod serde;

mod command;
mod option;
mod args;
mod arg_count;
mod command_line;
mod context;
mod error;
mod tokenizer;
mod parser;
mod parse_result;
mod symbol;

/// Utilities for provide suggestions.
pub mod suggestion;

/// Utilities for provide commands help information.
pub mod help;

// Re-exports
pub use self::command::*;
pub use self::option::*;
pub use self::args::*;
pub use self::arg_count::*;
pub use self::command_line::*;
pub use self::context::*;
pub use self::error::*;
pub use self::tokenizer::*;
pub use self::parser::*;
pub use self::parse_result::*;
pub use self::symbol::*;

#[macro_use]
mod app_macros;
pub use app_macros::*;

#[cfg(feature="macros")]
pub mod macros {
    extern crate clapi_macros;
    pub use clapi_macros::*;
}

#[cfg(feature="macros")]
pub use macros::*;

#[doc(hidden)]
pub mod macro_utils {
    pub extern crate clapi_macros_utils;

    #[doc(hidden)]
    #[macro_export]
    macro_rules! declare_option_var {
        ($options:ident, $name:ident: $ty:ty) => {
            $crate::macro_utils::clapi_macros_utils::__declare_option_var!($options, $name: $ty)
        };
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! declare_argument_var {
        ($arguments:ident, $name:ident: $ty:ty) => {
            $crate::macro_utils::clapi_macros_utils::__declare_argument_var!($arguments, $name: $ty)
        };
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! var_type {
        ($ty:ty) => {
            $crate::macro_utils::clapi_macros_utils::__var_type!($ty)
        };
    }
}

#[doc(hidden)]
pub use macro_utils::*;