use clapi::macros::*;

#[command]
#[arg(value, max=2)]
fn test(value: i64){}

fn main(){}