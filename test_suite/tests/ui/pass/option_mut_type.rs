use clapi::macros::*;

#[command]
#[option(number)]
fn test(mut number: u32){}

fn main(){}