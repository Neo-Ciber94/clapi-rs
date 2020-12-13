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


/// Utility methods and extensions intended for internal use.
#[macro_use]
pub mod utils;

mod command;
pub use command::*;

mod option;
pub use option::*;

mod args;
pub use args::*;

mod arg_count;
pub use arg_count::*;

mod command_line;
pub use command_line::*;

mod context;
pub use context::*;

mod error;
pub use error::*;

mod tokenizer;
pub use tokenizer::*;

mod parser;
pub use parser::*;

mod parse_result;
pub use parse_result::*;

mod suggestion;
pub use suggestion::*;

mod help;
pub use help::*;

mod symbol;
pub use symbol::*;

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

    #[macro_export]
    macro_rules! declare_option_var {
        ($options:ident, $name:ident: $ty:ty) => {
            $crate::macro_utils::clapi_macros_utils::__declare_option_var!($options, $name: $ty)
        };
    }

    #[macro_export]
    macro_rules! declare_argument_var {
        ($arguments:ident, $name:ident: $ty:ty) => {
            $crate::macro_utils::clapi_macros_utils::__declare_argument_var!($arguments, $name: $ty)
        };
    }

    #[macro_export]
    macro_rules! var_type {
        ($ty:ty) => {
            $crate::macro_utils::clapi_macros_utils::__var_type!($ty)
        };
    }
}

#[doc(hidden)]
pub use macro_utils::*;