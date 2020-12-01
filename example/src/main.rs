#![allow(dead_code)]
use clapi::macros::*;

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

#[command]
#[option(value, alias="v", description="The value", default="Hello World")]
fn main(value: String) {
    println!("{:?}", value);
}