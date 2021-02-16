use clapi::validator::parse_validator;
use clapi::{Argument, Command, CommandOption};
use std::num::NonZeroUsize;

fn main() -> clapi::Result<()> {
    let (context, result) = Command::new("math")
        .description("Provides a set of math operations")
        .version("1.0")
        // Subcommand for sum the numbers
        .subcommand(
            Command::new("sum")
                .description("Calculates the sum of the numbers")
                .arg(
                    Argument::one_or_more("values")
                        .description("Numbers to sum")
                        .validator(parse_validator::<f64>()),
                )
                .option(
                    CommandOption::new("pretty")
                        .description("Pretty output")
                        .alias("p"),
                )
                .option(
                    CommandOption::new("precision")
                        .requires_assign(true)
                        .arg(Argument::new()
                            .validator(parse_validator::<NonZeroUsize>())
                            .validation_error("`precision` expect a value greater than 0")),
                ),
        )
        // Subcommand for product the numbers
        .subcommand(
            Command::new("prod")
                .description("Calculates the product of the number")
                .arg(
                    Argument::one_or_more("values")
                        .description("Number to multiply")
                        .validator(parse_validator::<f64>()),
                )
                .option(
                    CommandOption::new("pretty")
                        .description("Pretty output")
                        .alias("p"),
                )
                .option(
                    CommandOption::new("precision")
                        .requires_assign(true)
                        .arg(Argument::new()
                            .validator(parse_validator::<NonZeroUsize>())
                            .validation_error("`precision` expect a value greater than 0")),
                ),
        )
        .parse_args_and_get_context()?;

    // Executing subcommand
    let subcommand = result.executing_command();

    // We check here if the subcommand is `sum` and `prod` to ensure is safe to get
    // `pretty`, `precision` and `values` which are share between those 2 subcommands
    if !matches!(subcommand.get_name(), "sum" | "prod") {
        // Shows a help message if the command is no `sum` or `prod`
        let mut buf = String::new();
        clapi::help::command_help(&mut buf, &context, subcommand, true);
        println!("{}", buf);
    } else {
        // Check if contains the `pretty` flag
        let pretty = result.options().contains("pretty");

        // Gets the precision of the operation or `None` if not set
        let precision = result
            .options()
            .get("precision")
            .map(|o| o.get_arg())
            .flatten()
            .map(|arg| arg.convert::<NonZeroUsize>().unwrap());

        // The numbers
        let values = result.get_arg("values").unwrap().convert_all::<f64>()?;

        // Check the executing subcommand name
        match subcommand.get_name() {
            "sum" => calculate(0_f64, values, pretty, '+', precision, |a, b| a + b),
            "prod" => calculate(1_f64, values, pretty, '*', precision, |a, b| a * b),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn calculate<F>(
    initial_value: f64,
    values: Vec<f64>,
    pretty: bool,
    operator: char,
    precision: Option<NonZeroUsize>,
    f: F,
) where
    F: FnMut(f64, &f64) -> f64,
{
    let result = values.iter().fold(initial_value, f);

    match precision {
        Some(prec) => {
            let precision = prec.get();

            if pretty {
                let expr: String = values
                    .iter()
                    .map(|s| format!("{:.1$}", s, precision))
                    .collect::<Vec<String>>()
                    .join(&format!(" {} ", operator));

                println!("{} = {:.2$}", expr, result, precision);
            } else {
                println!("{:.1$}", result, precision);
            }
        }
        None => {
            if pretty {
                let expr: String = values
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(&format!(" {} ", operator));

                println!("{} = {}", expr, result);
            } else {
                println!("{}", result);
            }
        }
    }
}
