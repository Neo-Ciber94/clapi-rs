use clapi::macros::*;

#[command]
#[option(x, min="abc")]
fn app(x: Vec<i64>){}

fn main(){}