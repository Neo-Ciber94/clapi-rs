use clapi::macros::*;

#[command]
#[option(slice)]
fn test(slice: &mut [u32]){}

fn main(){}