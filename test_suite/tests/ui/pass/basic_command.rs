use clapi::macros::*;
use clapi::{Context, Command};
use clapi::help::{Buffer, Help, DefaultHelp};

#[command(description="Prints a value", about="How to use the command", version=1.0)]
#[option(times, alias="t", arg="count", default=1, description="Times to repeat")]
#[arg(values, arg="text", min=1, max=100, description="Values to write")]
fn echo(times: u64, values: Vec<String>){
    #[subcommand(description=r#"Shows the author of the command"#, version=2.1)]
    fn author(){}

    #[help]
    static HELP : MyHelp = MyHelp;

    struct MyHelp;
    impl Help for MyHelp {
        fn help(&self, buf: &mut Buffer, context: &Context, command: &Command) -> std::fmt::Result{
            DefaultHelp::default().help(buf, context, command)
        }

        fn usage(&self, buf: &mut Buffer, context: &Context, command: &Command) -> std::fmt::Result{
            DefaultHelp::default().usage(buf, context, command)
        }
    }
}

fn main(){}