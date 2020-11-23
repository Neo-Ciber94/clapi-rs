use clapi::{Command, Argument, CommandOption, CommandLine, Options, ArgumentList};
use clapi::validator::parse_validator;
use clapi::Result;

fn main() -> Result<()> {
    let command = Command::root()
        .subcommand(Command::new("version")
            .handler(show_version(1, 0, 0)))
        .option(CommandOption::new("times")
            .alias("t")
            .arg(Argument::new("times")
                .validator(parse_validator::<i64>())
                .default(1)))
        .arg(Argument::new("values")
            .arg_count(1..)
            .validator(parse_validator::<i64>()))
        .handler(|opts, args|{
            let times = opts.get("times").unwrap().get_arg().unwrap().convert::<i64>()?;
            let values = args.get("values").unwrap().convert_all::<i64>()?;

            println!("total: {}", values.iter().sum::<i64>() * times);
            Ok(())
        });

    CommandLine::new(command)
        .use_default_suggestions()
        .use_default_help()
        .run()
}

fn show_version(mayor: u32, minor: u32, path: u32) -> impl FnMut(&Options, &ArgumentList) -> Result<()>{
    move |_, _| {
        println!("version {}.{}.{}", mayor, minor, path);
        Ok(())
    }
}