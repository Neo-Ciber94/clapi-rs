use clapi::validator::parse_validator;
use clapi::{Argument, Command, CommandOption};
use std::num::NonZeroUsize;

fn main() {
    let result = Command::new("repeater")
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
        .parse_args()
        .expect("failed to execute");

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
