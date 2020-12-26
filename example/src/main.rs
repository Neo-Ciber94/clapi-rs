use clapi::macros::*;
use std::sync::atomic::{AtomicI64, Ordering};
use clapi::help::{Help, DefaultHelp};
use clapi::{Context, Command};

mod count;
// mod utils;
mod data;

/// A command
#[command(description="A test", version=1)]
fn main(){
}

#[subcommand]
#[arg(values, min=1)]
fn echo(values: Vec<String>){
    for value in values {
        print!("{} ", value);
    }
    println!()
}

#[help]
static HELP : MyHelp = MyHelp;

struct MyHelp;
impl Help for MyHelp {
    fn help(&self, context: &Context, command: &Command) -> String {
        DefaultHelp::default().help(context, command)
    }

    fn usage(&self, context: &Context, command: &Command) -> String {
        DefaultHelp::default().usage(context, command)
    }
}