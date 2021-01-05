use clapi::macros::*;

#[command]
#[arg(value, min=2)]
fn test(value: Option<i64>){}

fn main(){}