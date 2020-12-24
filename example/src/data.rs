use super::*;

static VALUE : AtomicI64 = AtomicI64::new(0);

#[subcommand]
pub fn data(){}

#[subcommand]
pub fn get(){
    println!("{}", VALUE.load(Ordering::Relaxed));
}

#[subcommand(parent="data")]
#[arg(value)]
pub fn set(value: i64){
    VALUE.store(value, Ordering::Relaxed);
    get();
}