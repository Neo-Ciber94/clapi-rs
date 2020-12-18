use super::*;
use std::time::Duration;

#[subcommand]
#[arg(time)]
pub fn timer(time: u32){
    for current in 0..=time {
        println!("{}", time - current);
        std::thread::sleep(Duration::from_millis(500));
    }
}