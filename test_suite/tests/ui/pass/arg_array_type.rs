use clapi::macros::*;

#[command]
#[arg(array)]
fn test(array: [u32; 10]){}

fn main(){}