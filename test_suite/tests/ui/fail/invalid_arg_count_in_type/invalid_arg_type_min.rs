use clapi::macros::*;

#[command]
#[arg(value, min=0)]
fn test(value: i64){}

fn main(){}