use clapi::{Context, Command};
use clapi::macros::*;

#[command]
fn entry(){}

#[command_help]
fn custom_help(buf: &mut String, context: &Context, command: &Command, after_help_msg: bool) {
    // Forward
    clapi::help::command_help(buf, context, command, after_help_msg)
}

#[command_help]
fn other_help(buf: &mut String, context: &Context, command: &Command, after_help_msg: bool) {
    // Forward
    clapi::help::command_help(buf, context, command, after_help_msg)
}

fn main(){}