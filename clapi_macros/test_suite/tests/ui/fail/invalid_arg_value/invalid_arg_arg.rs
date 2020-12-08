use clapi::macros::*;

#[command]
#[arg(x, arg=123)]
fn app(x: i64){}

fn main(){}