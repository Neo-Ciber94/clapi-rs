use clapi::macros::*;

#[command]
fn app(){
    #[subcommand(help=1)]
    fn child(){}
}

fn main(){}