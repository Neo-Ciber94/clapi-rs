use clapi::macros::*;

#[command]
fn app(){
    #[subcommand(hidden=1)]
    fn child(){}
}

fn main(){}