use clapi::macros::*;

#[command]
#[arg(value, min=2, max=0)]
fn test(value: Vec<i64>){}

fn main(){}