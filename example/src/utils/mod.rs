use clapi_macros::*;
pub mod timer;

#[subcommand]
pub fn time(){
    println!("{:?}", std::time::SystemTime::now());
}