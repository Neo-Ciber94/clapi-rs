use clapi::macros::*;

#[command]
#[arg(x, error=123)]
fn app(x: i64){}

fn main(){}