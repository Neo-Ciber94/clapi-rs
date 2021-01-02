use clapi::macros::*;

#[command]
#[arg(values)]
#[arg(values, min=1)]
fn echo(values: Vec<String>){}

fn main(){}