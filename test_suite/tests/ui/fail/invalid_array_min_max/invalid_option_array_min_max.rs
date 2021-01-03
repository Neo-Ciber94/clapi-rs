use clapi::macros::*;

#[command]
#[option(values, min=0, max=3)]
fn test(values: [u32; 3]){}

fn main(){}