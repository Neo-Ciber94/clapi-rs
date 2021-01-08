use clapi::macros::*;

#[command]
#[option(x, hidden="xyz")]
fn app(x: i64){}

fn main(){}