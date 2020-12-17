use clapi::macros::*;

#[command]
#[arg(x, min="xyz")]
fn app(x: Vec<i64>){}

fn main(){}