use clapi::*;

// mod count;
// mod utils;

#[command]
#[option(arg="n", default=1)]
fn main(number: i64){
    println!("{}", number);
}
