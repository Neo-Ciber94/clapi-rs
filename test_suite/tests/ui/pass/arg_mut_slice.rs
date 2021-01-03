use clapi::macros::*;

#[command]
#[arg(slice)]
fn test(slice: &mut [u32]){}

fn main(){}