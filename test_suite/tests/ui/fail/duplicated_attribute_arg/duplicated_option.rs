use clapi::macros::*;

#[command]
#[option(times)]
#[option(times, default=1)]
fn echo(times: i64){}

fn main(){}