#![cfg_attr(doc_cfg, feature(doc_cfg))]
// #![feature(doc_cfg)]

//! # Clapi
//!
//! Clapi (Command-Line API) is a framework for create command line applications.
//!
//! Currently clapi provides several methods for create command line applications:
//! - Parsing the arguments
//! - Function handlers
//! - Macros
//! - Macros attributes
//!
//! See the examples below creating the same app using the 4 methods.
//!
//! ## Parsing the arguments
//! ```no_run
//! use clapi::{Command, CommandOption, Argument, CommandLine};
//! use clapi::validator::validate_type;
//! use std::num::NonZeroUsize;
//!
//! fn main() -> clapi::Result<()> {
//!     let command = Command::new("echo")
//!         .version("1.0")
//!         .description("outputs the given values on the console")
//!         .arg(Argument::one_or_more("values"))
//!         .option(
//!             CommandOption::new("times")
//!                 .alias("t")
//!                 .description("number of times to repeat")
//!                 .arg(
//!                     Argument::new()
//!                         .validator(validate_type::<NonZeroUsize>())
//!                         .validation_error("expected number greater than 0")
//!                         .default(NonZeroUsize::new(1).unwrap()),
//!                 ),
//!         ).handler(|opts, args| {
//!         let times = opts.convert::<usize>("times").unwrap();
//!         let values = args.get_raw_args()
//!             .map(|s| s.to_string())
//!             .collect::<Vec<String>>()
//!             .join(" ") as String;
//!
//!         for _ in 0..times {
//!             println!("{}", values);
//!         }
//!
//!         Ok(())
//!     });
//!
//!     CommandLine::new(command)
//!         .use_default_help()
//!         .use_default_suggestions()
//!         .run()
//!         .map_err(|e| e.exit())
//! }
//! ```
//!
//! ## Function handlers
//! ```no_run
//! use clapi::validator::validate_type;
//! use clapi::{Argument, Command, CommandLine, CommandOption};
//!
//! fn main() -> clapi::Result<()> {
//!     let command = Command::new("MyApp")
//!         .subcommand(
//!             Command::new("repeat")
//!                 .arg(Argument::one_or_more("values"))
//!                 .option(
//!                     CommandOption::new("times").alias("t").arg(
//!                         Argument::with_name("times")
//!                             .validator(validate_type::<u64>())
//!                             .default(1),
//!                     ),
//!                 )
//!                 .handler(|opts, args| {
//!                     let times = opts.get_arg("times").unwrap().convert::<u64>()?;
//!                     let values = args
//!                         .get("values")
//!                         .unwrap()
//!                         .convert_all::<String>()?
//!                         .join(" ");
//!                     for _ in 0..times {
//!                         println!("{}", values);
//!                     }
//!                     Ok(())
//!                 }),
//!         );
//!
//!     CommandLine::new(command)
//!         .use_default_suggestions()
//!         .use_default_help()
//!         .run()
//! }
//! ```
//! ## Macro
//!```no_run
//! use std::num::NonZeroUsize;
//!
//! fn main() -> clapi::Result<()> {
//!     let cli = clapi::app!{ echo =>
//!         (version => "1.0")
//!         (description => "outputs the given values on the console")
//!         (@option times =>
//!             (alias => "t")
//!             (description => "number of times to repeat")
//!             (@arg =>
//!                 (type => NonZeroUsize)
//!                 (default => NonZeroUsize::new(1).unwrap())
//!                 (error => "expected number greater than 0")
//!             )
//!         )
//!         (@arg values => (count => 1..))
//!         (handler (times: usize, ...args: Vec<String>) => {
//!             let values = args.join(" ");
//!             for _ in 0..times {
//!                 println!("{}", values);
//!             }
//!         })
//!     };
//!
//!     cli.use_default_suggestions()
//!         .use_default_help()
//!         .run()
//!         .map_err(|e| e.exit())
//! }
//!```
//!
//! ## Macro attributes
//! Requires `macros` feature enable.
//!
//! ```no_run compile_fail
//! use clapi::macros::*;
//! use std::num::NonZeroUsize;
//!
//! #[command(name="echo", description="outputs the given values on the console", version="1.0")]
//! #[arg(values, min=1)]
//! #[option(times,
//!     alias="t",
//!     description="number of times to repeat",
//!     default=1,
//!     error="expected number greater than 0"
//! )]
//! fn main(times: NonZeroUsize, values: Vec<String>) {
//!     let values = values.join(" ");
//!
//!     for _ in 0..times.get() {
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

/// Utilities for provide suggestions.
pub mod suggestion;

/// Utilities for provide commands help information.
pub mod help;

/// Representation of the command-line command, option and args.
pub mod token;

/// Converts the command-line arguments into tokens.
pub mod tokenizer;

/// Provides the `Validator` trait used for validate the values of an `Argument`.
pub mod validator;

/// Exposes the `struct Type` for arguments type checking.
#[cfg(feature = "typing")]
pub mod typing;

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

/// Clapi macros
#[macro_use]
mod app_macros;

/// Macro attributes for create command-line apps. Require `macros` feature enable.
#[cfg(feature = "macros")]
// #[doc(cfg(feature = "macros"))]
#[cfg_attr(doc_cfg, doc(cfg(feature = "macros")))]
pub mod macros {
    extern crate clapi_macros;
    pub use clapi_macros::*;
}

#[doc(hidden)]
#[cfg(feature = "macros")]
pub use macros::*;

/// Utilities intended for internal use.
#[doc(hidden)]
pub mod private {
    pub extern crate clapi_internal;

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
            $crate::private::clapi_internal::__declare_option_var!($options, $name: $ty)
        };
    }

    #[doc(hidden)]
    #[macro_export]
    macro_rules! declare_argument_var {
        ($arguments:ident, $name:ident: $ty:ty) => {
            $crate::private::clapi_internal::__declare_argument_var!($arguments, $name: $ty)
        };
    }
}
