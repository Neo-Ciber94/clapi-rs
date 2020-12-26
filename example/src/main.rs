use clapi::macros::*;
use clapi::{Command, CommandOption, Argument, CommandLine, Context};
use clapi::validator::parse_validator;
use clapi::help::{Help, DefaultHelp, Buffer};

//mod count;
// mod utils;
//mod data;

/// A command
#[command(description="A test", version=1)]
fn main(){

}

fn _entry(){
    let command = Command::root()
        .description("App to sum values")
        .arg(Argument::one_or_more("values")
            .description("Values to sum")
            .validator(parse_validator::<i64>()))
        .option(CommandOption::new("times")
            .description("Number of times to sum the values")
            .alias("t")
            .arg(Argument::new("times")
                .validator(parse_validator::<u64>())))
        .option(CommandOption::new("version").alias("v")
            .description("Shows the version"))
        .option(CommandOption::new("color")
            .description("Shows the output colored"))
        .option(CommandOption::new("datetime")
            .description("Shows the date and time with the output"))
        .option(CommandOption::new("just_time").alias("jt")
            .description("Shows the time with the output"))
        .subcommand(Command::new("version")
            .description("Shows the version"));

    CommandLine::new(command)
        .use_default_help()
        .use_default_suggestions()
        .run()
        .unwrap()
}

#[subcommand]
#[arg(values, min=1)]
fn echo(values: Vec<String>){
    for value in values {
        print!("{} ", value);
    }
    println!()
}

#[help]
static HELP : MyHelp = MyHelp;

struct MyHelp;
impl Help for MyHelp {
    fn help(&self, buf: &mut Buffer, context: &Context, command: &Command) -> std::fmt::Result {
        DefaultHelp::default().help(buf, context, command)
    }

    fn usage(&self, buf: &mut Buffer, context: &Context, command: &Command) -> std::fmt::Result {
        DefaultHelp::default().usage(buf, context, command)
    }
}