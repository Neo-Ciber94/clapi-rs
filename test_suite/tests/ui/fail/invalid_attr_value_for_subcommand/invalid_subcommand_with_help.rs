use clapi::macros::*;

#[command]
fn app(){
    #[subcommand(with_help=123)]
    fn child(){}
}

fn main(){}