use clapi::macros::*;

#[command]
#[option(x, flag=1)]
fn app(x: i64){}

fn main(){}