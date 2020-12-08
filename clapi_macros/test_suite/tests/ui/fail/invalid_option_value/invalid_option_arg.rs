use clapi::macros::*;

#[command]
#[option(x, arg=123)]
fn app(x: i64){}

fn main(){}