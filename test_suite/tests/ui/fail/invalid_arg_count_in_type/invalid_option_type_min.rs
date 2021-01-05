use clapi::macros::*;

#[command]
#[arg(option, min=0)]
fn test(value: i64){}

fn main(){}