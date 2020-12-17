use clapi::macros::*;

#[command]
#[arg(default=1)]
fn app(x: i64){}

fn main(){}