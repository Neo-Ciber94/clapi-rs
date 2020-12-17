use clapi::macros::*;

#[command]
fn app(){
    #[subcommand(version=true)]
    fn child(){}
}

fn main(){}