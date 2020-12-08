use clapi::macros::*;

#[command]
#[arg(x, min="abc")]
fn app(x: Vec<i64>){}

fn main(){}