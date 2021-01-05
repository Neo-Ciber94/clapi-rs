use clapi::macros::*;

#[command]
#[option(slice)]
fn test(slice: Vec<u32>){}

fn main(){}