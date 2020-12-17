use clapi::macros::*;

struct MyStruct;

impl MyStruct {
    #[command]
    fn app() {}
}

fn main(){}