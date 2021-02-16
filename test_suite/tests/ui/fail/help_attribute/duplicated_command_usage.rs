#![allow(unused_imports)]
use clapi::{Context, Command};
use clapi::macros::*;

#[command]
fn entry(){
    #[command_usage]
    fn custom_usage(buf: &mut String, context: &Context, command: &Command, after_help_msg: bool) {
        // Forward
        clapi::help::command_usage(buf, context, command, after_help_msg)
    }

    #[command_usage]
    fn other_usage(buf: &mut String, context: &Context, command: &Command, after_help_msg: bool) {
        // Forward
        clapi::help::command_usage(buf, context, command, after_help_msg)
    }
}

fn main(){}