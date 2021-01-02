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
//! use clapi::{Command, CommandOption, Argument, Parser, Context};
//! use clapi::help::{DefaultHelp, HelpKind, Help, Buffer};
//! use clapi::validator::parse_validator;
//!
//! let command = Command::root()
//!     .option(CommandOption::new("version").alias("v"))
//!     .subcommand(Command::new("repeat")
//!         .arg(Argument::one_or_more("values"))
//!         .option(CommandOption::new("times")
//!             .alias("t")
//!             .arg(Argument::new("times")
//!                 .validator(parse_validator::<u64>())
//!                 .default(1))));
//!
//! let context = Context::new(command);
//! let result = Parser.parse(&context, std::env::args().skip(1)).expect("unexpected error");
//!
//! if result.contains_option("version") {
//!     println!("MyApp 1.0");
//!     return;
//! }
//!
//! if result.command().get_name() == "repeat" {
//!     let times = result.get_option_arg("times")
//!         .unwrap()
//!         .convert::<u64>()
//!         .unwrap();
//!
//!     let values = result.arg().unwrap()
//!         .convert_all::<String>()
//!         .expect("not values specify")
//!         .join(" ");
//!
//!     for _ in 0..times {
//!         println!("{}", values);
//!     }
//! } else {
//!     // Fallthrough
//!     static HELP : DefaultHelp = DefaultHelp(HelpKind::Any);
//!
//!     let mut buffer = Buffer::new();
//!     HELP.help(&mut buffer, &context, result.command()).unwrap();
//!     println!("{}", buffer);
//! }
//! ```
//!
//! ## Function handlers
//! ```no_run
//! use clapi::validator::parse_validator;
//! use clapi::{Argument, Command, CommandLine, CommandOption};
//!
//! fn main() -> clapi::Result<()> {
//!     let command = Command::root()
//!         .option(CommandOption::new("version").alias("v"))
//!         .handler(|opts, _args| {
//!             if opts.contains("version") {
//!                 println!("MyApp 1.0");
//!             }
//!             Ok(())
//!         })
//!         .subcommand(
//!             Command::new("repeat")
//!                 .arg(Argument::one_or_more("values"))
//!                 .option(
//!                     CommandOption::new("times").alias("t").arg(
//!                         Argument::new("times")
//!                             .validator(parse_validator::<u64>())
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
//!```
//! fn main() -> clapi::Result<()> {
//!     let cli = clapi::app!{ =>
//!         (@option version => (alias => "v"))
//!         (handler () => println!("MyApp 1.0"))
//!         (@subcommand repeat =>
//!             (@arg values => (count => 1..))
//!             (@option times =>
//!                 (alias => "t")
//!                 (@arg times =>
//!                     (type => u64)
//!                     (default => 1)
//!                     (count => 1)
//!                 )
//!             )
//!             (handler (times: u64, ...values: Vec<String>) => {
//!                 let values = values.join(" ");
//!                 for _ in 0..times {
//!                     println!("{}", values);
//!                 }
//!             })
//!         )
//!     };
//!
//!      cli.use_default_help()
//!         .use_default_suggestions()
//!         .run()
//! }
//!```
//!
//! ## Macro attributes
//! Requires `macros` feature enable.
//!
//! ```compile_fail
//! use clapi::macros::*;
//!
//! #[command(version=1.0)]
//! fn main(){ }
//!
//! #[subcommand]
//! #[option(times, alias="t", default=1)]
//! #[arg(values, min=1)]
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

/// Clapi macros
#[macro_use]
mod app_macros;

/// Macro attributes for create command-line apps. Require `macros` feature enable.
#[cfg(feature = "macros")]
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