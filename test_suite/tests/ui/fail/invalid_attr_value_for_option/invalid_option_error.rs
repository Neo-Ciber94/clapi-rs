use clapi::macros::*;

#[command]
#[option(x, error=123)]
fn app(x: i64){}

fn main(){}