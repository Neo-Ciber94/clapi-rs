use clapi::macros::*;

#[command]
#[option(x, name=123)]
fn app(x: i64){}

fn main(){}