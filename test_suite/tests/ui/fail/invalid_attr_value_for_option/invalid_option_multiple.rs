use clapi::macros::*;

#[command]
#[option(x, multiple="xyz")]
fn app(x: i64){}

fn main(){}