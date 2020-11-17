use super::*;
pub mod timer;

#[subcommand]
pub fn time(){
    println!("{:?}", std::time::SystemTime::now());
}