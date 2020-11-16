use clapi_macros::*;
use std::time::Duration;

#[subcommand]
#[arg(name="time")]
pub fn timer(time: u32){
    for current in 0..=time {
        println!("{}", time - current);
        std::thread::sleep(Duration::from_millis(500));
    }
}