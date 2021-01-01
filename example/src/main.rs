use clapi::macros::*;
// use clapi::help::{Help, DefaultHelp, Buffer};
// use clapi::{Context, Command};

// mod count;
// mod utils;
// mod data;

#[command]
pub fn main(){
}

#[subcommand]
#[arg(values, min=1)]
fn echo(values: Vec<String>){
    let mut iter = values.iter().peekable();
    while let Some(x) = iter.next() {
        print!("{}", x);

        if iter.peek().is_some() {
            print!(" ");
        }
    }

    println!()
}

// fn _entry(){
//     let command = Command::root()
//         .description("App to sum values")
//         .arg(Argument::one_or_more("values")
//             .description("Values to sum")
//             .validator(parse_validator::<i64>()))
//         .option(CommandOption::new("times")
//             .description("Number of times to sum the values")
//             .alias("t")
//             .arg(Argument::new("times")
//                 .validator(parse_validator::<u64>())))
//         .option(CommandOption::new("version").alias("v")
//             .description("Shows the version"))
//         .option(CommandOption::new("color")
//             .description("Shows the output colored"))
//         .option(CommandOption::new("datetime")
//             .description("Shows the date and time with the output"))
//         .option(CommandOption::new("just_time").alias("jt")
//             .description("Shows the time with the output"))
//         .subcommand(Command::new("version")
//             .description("Shows the version"));
//
//     CommandLine::new(command)
//         .use_default_help()
//         .use_default_suggestions()
//         .run()
//         .unwrap()
// }