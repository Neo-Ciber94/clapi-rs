use super::*;

static mut VALUE : i64 = 0;

#[subcommand]
pub fn data(){
}

#[subcommand(parent="data")]
pub fn get(){
    unsafe {
        println!("{}", VALUE);
    }
}

#[subcommand(parent="data")]
#[arg(value)]
pub fn set(value: i64){
    unsafe {
        VALUE = value;
        get();
    }
}