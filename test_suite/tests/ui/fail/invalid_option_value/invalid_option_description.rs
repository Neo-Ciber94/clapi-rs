use clapi::macros::*;

#[command]
#[option(x, description=1)]
fn app(x: i64){}

fn main(){}