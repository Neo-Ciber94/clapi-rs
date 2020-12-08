use clapi::macros::*;

#[command]
fn app(){
    #[subcommand(about=1)]
    fn child(){}
}

fn main(){}