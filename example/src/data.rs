use super::*;
use std::sync::atomic::{AtomicI64, Ordering};

static VALUE : AtomicI64 = AtomicI64::new(0);

#[subcommand(parent="echo")]
pub fn data(){}

#[subcommand(parent="data")]
pub fn get(){
    println!("{}", VALUE.load(Ordering::Relaxed));
}

#[subcommand(parent="data")]
#[arg(value)]
pub fn set(value: i64){
    VALUE.store(value, Ordering::Relaxed);
    get();
}