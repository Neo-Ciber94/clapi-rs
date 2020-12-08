use clapi::macros::*;

#[command]
#[arg(x, description=1)]
fn app(x: i64){}

fn main(){}