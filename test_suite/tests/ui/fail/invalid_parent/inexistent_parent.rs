use clapi::macros::*;

#[command]
fn files(){
    #[subcommand]
    fn list(){}

    #[subcommand(parent="list2")]
    fn sort(){}
}

fn main(){}