use clapi::macros::*;

#[command]
#[arg(x, name=123)]
fn app(x: i64){}

fn main(){}