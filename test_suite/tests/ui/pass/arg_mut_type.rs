use clapi::macros::*;

#[command]
#[arg(number)]
fn test(mut number: u32){}

fn main(){}