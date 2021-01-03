use clapi::macros::*;

#[command]
#[arg(slice)]
fn test(slice: &[u32]){}

fn main(){}