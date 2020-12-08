use clapi::macros::*;

#[command]
#[arg(x, default="10")]
fn app(x: i64){}

fn main(){}