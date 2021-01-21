use clapi::macros::*;

#[command]
fn app(){
    #[subcommand(with_usage=123)]
    fn child(){}
}

fn main(){}