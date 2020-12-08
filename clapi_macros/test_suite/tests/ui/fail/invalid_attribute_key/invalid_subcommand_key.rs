use clapi::macros::*;

#[command]
fn app(){
    #[subcommand(abc="hello")]
    fn child(){}
}

fn main(){}