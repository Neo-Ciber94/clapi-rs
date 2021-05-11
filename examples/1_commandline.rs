use clapi::{Command, CommandOption, Argument, CommandLine};
use clapi::validator::validate_type;
use std::num::NonZeroUsize;

fn main() -> clapi::Result<()> {
    let command = Command::new("echo")
        .version("1.0")
        .description("outputs the given values on the console")
        .arg(Argument::one_or_more("values"))
        .option(
            CommandOption::new("times")
                .alias("t")
                .description("number of times to repeat")
                .arg(
                    Argument::new()
                        .validator(validate_type::<NonZeroUsize>())
                        .validation_error("expected number greater than 0")
                        .default(NonZeroUsize::new(1).unwrap()),
                ),
        ).handler(|opts, args| {
        let times = opts.convert::<usize>("times").unwrap();
        let values = args.get_raw_args()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(" ") as String;

        for _ in 0..times {
            println!("{}", values);
        }

        Ok(())
    });

    CommandLine::new(command)
        .use_default_help()
        .use_default_suggestions()
        .run()
        .map_err(|e| e.exit())
}