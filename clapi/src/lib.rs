pub mod utils;

pub mod arg_count;
pub mod args;
#[allow(dead_code)]
pub mod command;
pub mod command_line;
pub mod context;
pub mod error;
pub mod help;
pub mod option;
pub mod parse_result;
pub mod parser;
pub mod root_command;
pub mod suggestion;
pub mod symbol;
pub mod tokenizer;

#[cfg(feature="macros")]
pub mod macros {
    extern crate clapi_macros;
    pub use clapi_macros::*;
}

#[cfg(feature="macros")]
pub use macros::*;