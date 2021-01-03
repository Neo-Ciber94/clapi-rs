use clapi::macros::*;

#[command]
#[option(slice)]
fn test(slice: &[u32]){}

fn main(){}