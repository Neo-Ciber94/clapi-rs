#![allow(dead_code)]
use clapi::macros::*;
use clapi::{DefaultParser, Parser, Context, Command};

// mod count;
// mod utils;

#[subcommand(description="Prints a value to the console")]
#[option(times, alias="t", default=1)]
#[arg(values)]
fn echo(times: usize, values: Vec<String>) {
    for _ in 0..times {
        for value in &values {
            print!("{} ", value);
        }

        println!();
    }
}

#[command(version=1.0)]
#[option(value, description="The value", default="Hello World")]
fn main(value: String) -> clapi::Result<()> {
    println!("{:?}", value);
    Ok(())
}

fn entry(){
    let context = Context::new(Command::root());
    let result = DefaultParser.parse(&context, vec!["args", "one", "two"]);
}