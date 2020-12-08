use clapi::macros::*;

#[command]
#[option(number, abc="hello")]
fn app(number: String){}

fn main(){}