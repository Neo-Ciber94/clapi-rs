use clapi::macros::*;

#[command]
fn app(){
    #[subcommand(description=1)]
    fn child(){}
}

fn main(){}