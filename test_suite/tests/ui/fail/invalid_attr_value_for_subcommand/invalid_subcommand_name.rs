use clapi::macros::*;

#[command]
fn app(){
    #[subcommand(name=123)]
    fn child(){}
}

fn main(){}