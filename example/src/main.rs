use clapi::validator::parse_validator;
use clapi::*;

// mod count;
// mod utils;

fn main() {
    let command = Command::new("MyApp")
        .description("An app to sum numbers")
        .arg(
            Argument::one_or_more("numbers")
                .description("Numbers to sum")
                .validator(parse_validator::<i64>()),
        )
        .option(
            CommandOption::new("times")
                .description("Times to sum the numbers")
                .arg(Argument::new("times").validator(parse_validator::<u64>())),
        )
        .option(
            CommandOption::new("add")
                .description("Number to add")
                .arg(Argument::new("number").validator(parse_validator::<i64>())),
        )
        .option(
            CommandOption::new("sub")
                .description("Number to sub")
                .arg(Argument::new("number").validator(parse_validator::<i64>())),
        )
        .subcommand(Command::new("version").description("Shows the version of the command"));


    println!("{:#?}", Command::root());

    //println!("{}", serde_json::to_string_pretty(&command).unwrap());
    //println!("{}", serde_yaml::to_string(&command).unwrap());
}
