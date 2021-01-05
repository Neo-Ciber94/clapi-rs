use clapi::macros::*;

#[command]
#[arg(slice)]
fn test(slice: Vec<u32>){}

fn main(){}