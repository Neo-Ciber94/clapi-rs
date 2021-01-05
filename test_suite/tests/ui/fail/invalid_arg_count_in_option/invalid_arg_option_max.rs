use clapi::macros::*;

#[command]
#[arg(value, max=2)]
fn test(value: Option<usize>){}

fn main(){}