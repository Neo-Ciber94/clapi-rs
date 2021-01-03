use clapi::macros::*;

#[command]
#[arg(number)]
fn test(number: Option<u32>){}

fn main(){}