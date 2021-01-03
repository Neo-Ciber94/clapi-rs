use clapi::macros::*;

#[command]
#[option(number)]
fn test(number: Option<u32>){}

fn main(){}