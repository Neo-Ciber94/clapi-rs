use clapi::validator::validate_type;
use clapi::{Argument, Command, CommandOption};
use std::num::NonZeroUsize;

fn main() {
    let result = Command::new("echo")
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
        )
        .parse_args()
        .map_err(|e| e.exit())
        .unwrap();

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
