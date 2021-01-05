use clapi::macros::*;

#[command]
#[arg(x, values="1", "2", "3")]
fn app(x: Vec<i64>){}

fn main(){}