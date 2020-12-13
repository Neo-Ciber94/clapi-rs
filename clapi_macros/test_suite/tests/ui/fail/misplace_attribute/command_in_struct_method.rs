use clapi::macros::*;

struct MyStruct;
impl MyStruct {
    #[command]
    fn app(&self) {}
}

fn main(){}