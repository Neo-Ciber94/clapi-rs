use clapi::macros::*;

#[command]
#[option(value, min=2)]
fn test(value: Option<i64>){}

fn main(){}