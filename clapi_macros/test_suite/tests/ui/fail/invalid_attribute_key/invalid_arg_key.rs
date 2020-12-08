use clapi::macros::*;

#[command]
#[arg(number, abc="hello")]
fn app(number: String){}

fn main(){}