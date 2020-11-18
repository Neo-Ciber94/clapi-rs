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
//! use clapi::RootCommand;
//! use clapi::CommandOption;
//! use clapi::Command;
//! use clapi::Arguments;
//! use clapi::{DefaultParser, Parser};
//! use clapi::Context;
//! use std::env::args;
//!
//! // First we define a root command with all its options, subcommands and args.
//! let command = RootCommand::new()
//!     .set_option(CommandOption::new("version").set_alias("v"))
//!     .set_command(Command::new("repeat")
//!         .set_args(Arguments::one_or_more())
//!         .set_option(CommandOption::new("times").set_alias("t")
//!             .set_args(Arguments::zero_or_one()
//!                 .set_default_values(&[1]))));
//!
//! // The context contains the the `root`, prefixes and delimiters.
//! let context = Context::new(command);
//!
//! // Parse the `args` we skip the first value which can be the executable path
//! let result = DefaultParser.parse(&context, args().skip(1));
//!
//! match result {
//!     Ok(parse_result) => {
//!         if parse_result.contains_option("version"){
//!             println!("version 1.0");
//!         } else if parse_result.command().name() == "repeat" {
//!             // This will panic if the arg is no a `u32`
//!             let times = parse_result.get_option_arg_as::<u32>("times").unwrap().unwrap();
//!             // Get all the arguments
//!             let values = parse_result.args().values().join(" ");
//!             for _ in 0..times {
//!                 println!("{}", values)
//!             }
//!
//!         } else {
//!             unreachable!()
//!         }
//!     }
//!     Err(error) => {
//!         panic!("{}", error);
//!     }
//! }
//! ```
//!
//! ## Function handlers
//! ```no_run
//! use clapi::RootCommand;
//! use clapi::CommandOption;
//! use clapi::Command;
//! use clapi::Arguments;
//!
//! // First we define a root command with all its options, subcommands and args.
//! let command = RootCommand::new()
//!     .set_option(CommandOption::new("version").set_alias("v"))
//!     // We define a handler for the `root` command
//!     .set_handler(|opts, args| {
//!         if opts.contains("version"){
//!             println!("version 1.0");
//!         }
//!         Ok(())
//!     })
//!     .set_command(Command::new("repeat")
//!         .set_args(Arguments::one_or_more())
//!         .set_option(CommandOption::new("times").set_alias("t")
//!             .set_args(Arguments::zero_or_one()
//!                 .set_default_values(&[1])))
//!     // We define a handler for the `repeat` command
//!     .set_handler(|opts,args|{
//!         let times = opts.get_arg_as::<u32>("times").unwrap()?;
//!         let values = args.values().join(" ");
//!         for _ in 0..times {
//!             println!("{}", values);
//!         }
//!         Ok(())
//!     }));
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
//! #[command(version="version 1.0")]
//! fn main(){ }
//!
//! // Mark a function as a `subcommand` and defines if `option` and `arg`
//! #[subcommand]
//! #[option(name="times", alias="t", default=1)]
//! #[arg(name="values")]
//! fn repeat(times: u32, values: Vec<String>){
//!     let values = values.join(" ");
//!     for _ in 0..times {
//!         println!("{}", values);
//!     }
//! }
//! ```


/// Utility methods and extensions intended for internal use.
pub mod utils;

mod command;
pub use command::*;

mod root_command;
pub use root_command::*;

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

#[cfg(feature="macros")]
pub mod macros {
    extern crate clapi_macros;
    pub use clapi_macros::*;
}

#[cfg(feature="macros")]
pub use macros::*;