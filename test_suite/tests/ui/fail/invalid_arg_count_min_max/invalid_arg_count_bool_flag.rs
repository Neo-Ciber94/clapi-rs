use clapi::macros::*;

#[command]
#[arg(enable, min=0, max=1)]
fn test(enable: bool){}

fn main(){}