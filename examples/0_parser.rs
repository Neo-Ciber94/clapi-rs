use clapi::validator::parse_validator;
use clapi::{get_help_message, Argument, Command, CommandOption, MessageKind};
use std::num::NonZeroUsize;

fn main() {
    let (context, result) = Command::new("repeater")
        .version("1.0")
        .description("repeat the given output")
        .arg(Argument::one_or_more("values"))
        .option(
            CommandOption::new("times")
                .alias("t")
                .description("number of times to repeat")
                .arg(
                    Argument::new()
                        .validator(parse_validator::<NonZeroUsize>())
                        .validation_error("expected number greater than 0")
                        .default(NonZeroUsize::new(1).unwrap()),
                ),
        )
        .parse_args_and_get_context()
        .expect("failed to execute");

    if result.command_name() == "help" || result.options().contains("help") {
        let arg = if result.command_name() == "help" {
            result.arg()
        } else {
            result.options().get("help").unwrap().get_arg()
        };

        println!("{}",
            get_help_message(
                &context,
                arg.map(|s| s.get_values()),
                MessageKind::Help
            ).unwrap()
        );
    } else if result.options().contains("version") {
        println!("{} {}", result.command_name(), result.command_version().unwrap());
    } else {

        let times = result
            .options()
            .convert::<NonZeroUsize>("times")
            .unwrap()
            .get();

        let values = result.arg().unwrap().get_values().join(" ") as String;
        for _ in 0..times {
            println!("{}", values)
        }
    }
}
