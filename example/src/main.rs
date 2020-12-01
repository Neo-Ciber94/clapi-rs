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
fn main(enable: Option<bool>, numbers: Vec<i64>) {
    println!("enable: {:?}, numbers: {:?}", enable, numbers)
}
