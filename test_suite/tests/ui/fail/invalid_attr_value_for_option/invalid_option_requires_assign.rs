use clapi::macros::*;

#[command]
#[option(x, requires_assign="xyz")]
fn app(x: i64){}

fn main(){}