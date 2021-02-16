use clapi::{Context, Command};
use clapi::macros::*;

#[command]
fn entry(){}

#[command_help]
fn custom_help(buf: &mut String, context: &Context, command: &Command) {
    // Forward
    clapi::help::command_help(buf, context, command)
}

#[command_help]
fn other_help(buf: &mut String, context: &Context, command: &Command) {
    // Forward
    clapi::help::command_help(buf, context, command)
}

fn main(){}