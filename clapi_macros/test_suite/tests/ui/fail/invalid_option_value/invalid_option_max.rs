use clapi::macros::*;

#[command]
#[option(x, max="xyz")]
fn app(x: Vec<i64>){}

fn main(){}