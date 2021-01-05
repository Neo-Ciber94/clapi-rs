use clapi::macros::*;

#[command]
#[option(value, max=2)]
fn test(value: Option<usize>){}

fn main(){}