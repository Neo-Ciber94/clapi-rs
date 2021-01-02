use clapi::macros::*;

#[command]
#[option(enable)]
#[arg(enable)]
fn echo(enable: bool){}

fn main(){}