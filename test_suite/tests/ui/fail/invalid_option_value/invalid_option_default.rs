use clapi::macros::*;

#[command]
#[option(x, default="hello")]
fn app(x: i64){}

fn main(){}