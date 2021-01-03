use clapi::macros::*;

#[command]
#[option(array)]
fn test(array: [u32; 10]){}

fn main(){}