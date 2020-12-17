use clapi::macros::*;

#[command]
#[option(x, alias=1)]
fn app(x: i64){}

fn main(){}