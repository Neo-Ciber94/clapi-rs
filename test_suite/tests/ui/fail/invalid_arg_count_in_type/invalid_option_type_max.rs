use clapi::macros::*;

#[command]
#[option(value, max=2)]
fn test(value: i64){}

fn main(){}