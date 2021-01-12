use clapi::macros::*;

#[command]
fn app(){
    #[subcommand(usage=1)]
    fn child(){}
}

fn main(){}