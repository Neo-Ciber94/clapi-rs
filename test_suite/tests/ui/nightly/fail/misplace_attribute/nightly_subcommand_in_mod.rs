use clapi::macros::*;

#[command]
fn app(){}

pub mod module {
    use clapi::macros::*;

    #[subcommand]
    pub fn child(){}
}

fn main(){}