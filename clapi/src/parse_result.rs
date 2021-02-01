use crate::args::ArgumentList;
use crate::command::Command;
use crate::option::{CommandOption, OptionList};
use crate::Argument;

/// Represents the result of a parse operation
/// and provides a set of methods to query over the values.
#[derive(Debug, Clone)]
pub struct ParseResult {
    command: Command,
    options: OptionList,
    args: ArgumentList,
}

impl ParseResult {
    /// Constructs a new `ParseResult`.
    pub fn new(command: Command, options: OptionList, args: ArgumentList) -> Self {
        ParseResult {
            command,
            options,
            args,
        }
    }

    /// Returns the executing `Command`.
    pub fn executing_command(&self) -> &Command {
        &self.command
    }

    /// Returns the `Options` passed to the executing command.
    pub fn options(&self) -> &OptionList {
        &self.options
    }

    /// Returns the `Argument` passed to the executing command or `None` is there is more than 1 argument.
    pub fn arg(&self) -> Option<&Argument> {
        if self.args.len() == 1 {
            Some(&self.args[0])
        } else {
            None
        }
    }

    /// Returns the `Argument`s passed to the executing command.
    pub fn args(&self) -> &ArgumentList {
        &self.args
    }

    /// Returns `true` if the executing command contains an option with the given name or alias.
    pub fn contains_option<S: AsRef<str>>(&self, name_or_alias: S) -> bool {
        self.options.contains(name_or_alias)
    }

    /// Returns `true` if the executing command contains an argument with the given name.
    pub fn contains_arg<S: AsRef<str>>(&self, arg_name: S) -> bool {
        self.args.contains(arg_name)
    }

    /// Returns the `Argument` with the given name or `None` if not found.
    pub fn get_arg<S: AsRef<str>>(&self, arg_name: S) -> Option<&Argument> {
        self.args.get(arg_name)
    }

    /// Returns the `CommandOption` with the given name or alias, or `None`.
    pub fn get_option<S: AsRef<str>>(&self, name_or_alias: S) -> Option<&CommandOption> {
        self.options.get(name_or_alias)
    }

    /// Returns the single argument of the option with the given name or alias,
    /// or `None` if not found or if contains more than one argument.
    pub fn get_option_arg<S: AsRef<str>>(&self, name_or_alias: S) -> Option<&Argument> {
        self.get_option(name_or_alias)
            .map(|o| o.get_arg())
            .flatten()
    }

    /// Returns the arguments of the option with the given name or alias, or `None` if not found.
    pub fn get_option_args<S: AsRef<str>>(&self, name_or_alias: S) -> Option<&ArgumentList> {
        self.get_option(name_or_alias).map(|o| o.get_args())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::validator::parse_validator;
    use crate::{split_into_args, Context, Parser, ErrorKind};

    fn parse_with(value: &str, command: Command) -> crate::Result<ParseResult> {
        let context = Context::new(command);
        Parser::new(&context).parse(split_into_args(value))
    }

    #[test]
    fn parse_result_command_test() {
        let command = Command::new("MyApp")
            .arg(Argument::one_or_more("values"))
            .option(
                CommandOption::new("repeat").alias("r").arg(
                    Argument::with_name("times")
                        .validator(parse_validator::<u64>())
                        .default(1),
                ),
            )
            .option(
                CommandOption::new("color")
                    .alias("c")
                    .arg(Argument::with_name("color").valid_values(&["red", "blue", "green"])),
            );

        let result = parse_with("--repeat 2 -c red hello world!", command.clone()).unwrap();
        assert!(result.contains_option("repeat"));
        assert!(result.contains_option("r"));
        assert!(result.contains_option("color"));
        assert!(result.contains_option("c"));
        assert_eq!(
            result
                .get_option("repeat")
                .unwrap()
                .get_arg()
                .unwrap()
                .convert::<u64>()
                .ok(),
            Some(2)
        );
        assert!(result
            .get_option("color")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("red"));
        assert_eq!(result.arg().unwrap().get_values(), &["hello", "world!"]);

        // Whitespace test
        assert!(parse_with("\" \"", command.clone())
            .unwrap()
            .arg()
            .unwrap()
            .contains(" "));

        // Ok
        assert!(parse_with("Hola Mundo!", command.clone()).is_ok());
        assert!(parse_with("--repeat 10 Hola Mundo!", command.clone()).is_ok());
        assert!(parse_with("--color green Hola Mundo!", command.clone()).is_ok());
        assert!(parse_with("-c blue -- Hola Mundo!", command.clone()).is_ok());
        assert!(parse_with("\" \"", command.clone()).is_ok());

        // Err
        assert!(parse_with("--repeat -2 Hola Mundo!", command.clone()).is_err());
        assert!(parse_with("", command.clone()).is_err());
        assert!(parse_with("--repeat 2 -c:red", command.clone()).is_err());
        assert!(parse_with("-r a Hello World", command.clone()).is_err());
        assert!(parse_with("-c yellow Hello World", command.clone()).is_err());
    }

    #[test]
    fn parse_result_subcommand_test() {
        let command = Command::new("MyApp")
            .subcommand(Command::new("version"))
            .subcommand(
                Command::new("set")
                    .arg(Argument::one_or_more("value"))
                    .option(CommandOption::new("repeat").arg(Argument::with_name("count"))),
            )
            .subcommand(Command::new("get"));

        let result = parse_with("set --repeat 1 1 2 3 4", command.clone()).unwrap();
        assert_eq!(result.executing_command().get_name(), "set");
        assert!(result
            .get_option("repeat")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("1"));
        assert_eq!(
            result.arg().unwrap().get_values(),
            &[
                "1".to_owned(),
                "2".to_owned(),
                "3".to_owned(),
                "4".to_owned()
            ]
        );

        // Ok
        assert!(parse_with("version", command.clone()).is_ok());
        assert!(parse_with("set 1 2 3", command.clone()).is_ok());
        assert!(parse_with("get", command.clone()).is_ok());
        assert!(parse_with("", command.clone()).is_ok());

        // Err
        assert!(parse_with("Hello", command.clone()).is_err());
        assert!(parse_with("version hello", command.clone()).is_err());
        assert!(parse_with("version set 1 2 3", command.clone()).is_err());
        assert!(parse_with("version hello", command.clone()).is_err());
    }

    #[test]
    fn parse_result_required_option_test() {
        let command = Command::new("MyApp")
            .option(CommandOption::new("enable"))
            .option(
                CommandOption::new("times")
                    .alias("t")
                    .required(true)
                    .arg(Argument::with_name("times").validator(parse_validator::<u64>())),
            )
            .arg(Argument::zero_or_more("values"));

        let result = parse_with("--times 1 -- one two three", command.clone()).unwrap();
        assert!(result.contains_option("times"));
        assert!(result
            .get_option("times")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("1"));
        assert!(result.arg().unwrap().contains("one"));
        assert!(result.arg().unwrap().contains("two"));
        assert!(result.arg().unwrap().contains("three"));

        // Ok
        assert!(parse_with("--times 1", command.clone()).is_ok());
        assert!(parse_with("--times 1 1 2 3", command.clone()).is_ok());
        assert!(parse_with("--times 1 --enable", command.clone()).is_ok());

        // This is `Ok` because `--enable` takes not arguments
        // so the value `false` is passed as command argument
        assert!(parse_with("--times 1 --enable false", command.clone()).is_ok());

        // Err

        // Unlike above here is an error due `false` is being passed to `--enable` as argument
        // but it takes not arguments
        // Any tokens before `--` are considered arguments to the previous option
        // and the values following `--` are considered arguments to the command
        assert!(parse_with("--times 1 --enable false --", command.clone()).is_err());

        assert!(parse_with(" ", command.clone()).is_err());
        assert!(parse_with("--times", command.clone()).is_err());
    }

    #[test]
    fn parse_result_options_test() {
        let command = Command::new("MyApp")
            .option(CommandOption::new("hour").alias("h"))
            .option(CommandOption::new("minute").alias("m"))
            .option(CommandOption::new("second").alias("s"))
            .option(
                CommandOption::new("enable").arg(
                    Argument::with_name("value")
                        .validator(parse_validator::<bool>())
                        .values_count(0..=1),
                ),
            );

        let result = parse_with("--hour -m -s --enable false", command.clone()).unwrap();
        assert_eq!(result.args().len(), 0);
        assert!(result.contains_option("hour"));
        assert!(result.contains_option("minute"));
        assert!(result.contains_option("second"));
        assert!(result
            .get_option("enable")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("false"));
    }

    #[test]
    fn parse_result_multiple_args_test() {
        let command = Command::new("MyApp")
            .arg(Argument::with_name("min").validator(parse_validator::<i64>()))
            .arg(Argument::with_name("max").validator(parse_validator::<i64>()))
            .option(
                CommandOption::new("replace")
                    .alias("r")
                    .arg(Argument::with_name("from"))
                    .arg(Argument::with_name("to")),
            );

        let result = parse_with("--replace a A -- 2 10", command.clone()).unwrap();
        assert!(result
            .get_option("replace")
            .unwrap()
            .get_args()
            .get("from")
            .unwrap()
            .contains("a"));
        assert!(result
            .get_option("replace")
            .unwrap()
            .get_args()
            .get("to")
            .unwrap()
            .contains("A"));
        assert_eq!(
            result.get_arg("min").unwrap().convert::<i64>().ok(),
            Some(2)
        );
        assert_eq!(
            result.get_arg("max").unwrap().convert::<i64>().ok(),
            Some(10)
        );

        // Ok
        assert!(parse_with("2 10", command.clone()).is_ok());

        // Err
        assert!(parse_with("--replace hello HELLO", command.clone()).is_err());
        assert!(parse_with("25", command.clone()).is_err());
    }

    #[test]
    fn parse_result_eoa_test() {
        let command = Command::new("MyApp")
            .arg(Argument::one_or_more("args"))
            .option(CommandOption::new("A").alias("a"))
            .option(CommandOption::new("B").alias("b"))
            .option(CommandOption::new("C").alias("c"))
            .option(CommandOption::new("D").alias("d").arg(Argument::one_or_more("d")));

        let result1 = parse_with("--A --B -- --C", command.clone()).unwrap();
        assert_eq!(result1.options().len(), 2);
        assert_eq!(result1.arg().unwrap().get_values().len(), 1);
        assert!(result1.contains_option("A"));
        assert!(result1.contains_option("B"));
        assert!(result1.arg().unwrap().contains("--C"));

        let result2 = parse_with("-- --A -b --C", command.clone()).unwrap();
        assert_eq!(result2.options().len(), 0);
        assert_eq!(result2.arg().unwrap().get_values().len(), 3);
        assert!(result2.arg().unwrap().contains("--A"));
        assert!(result2.arg().unwrap().contains("-b"));
        assert!(result2.arg().unwrap().contains("--C"));

        let result3 = parse_with("-- -- -a -b -c", command.clone()).unwrap();
        assert_eq!(result3.options().len(), 0);
        assert_eq!(result3.arg().unwrap().get_values().len(), 4);
        assert!(result3.arg().unwrap().contains("--"));
        assert!(result3.arg().unwrap().contains("-a"));
        assert!(result3.arg().unwrap().contains("-b"));
        assert!(result3.arg().unwrap().contains("-c"));

        let result4 = parse_with("--D 1 2 3 -- hello world", command.clone()).unwrap();
        assert_eq!(result4.options().len(), 1);
        assert!(result4.get_option_arg("D").unwrap().contains("1"));
        assert!(result4.get_option_arg("D").unwrap().contains("2"));
        assert!(result4.get_option_arg("D").unwrap().contains("3"));
        assert!(result4.arg().unwrap().contains("hello"));
        assert!(result4.arg().unwrap().contains("world"));

        let result5 = parse_with("1 2 3 -- hello world", command.clone());
        assert!(result5.is_err());
    }

    #[test]
    fn parse_result_variable_arg_count_test1() {
        let command = Command::new("MyApp")
            .arg(Argument::with_name("values").values_count(0..=3))
            .option(
                CommandOption::new("letters").arg(
                    Argument::with_name("letters")
                        .values_count(1..)
                        .validator(parse_validator::<char>()),
                ),
            )
            .option(
                CommandOption::new("numbers").arg(
                    Argument::with_name("numbers")
                        .values_count(1..=2)
                        .validator(parse_validator::<i64>()),
                ),
            );

        let result = parse_with(
            "--letters a b c d e --numbers 1 -- one two three",
            command.clone(),
        )
        .unwrap();

        assert_eq!(
            result
                .get_option("letters")
                .unwrap()
                .get_arg()
                .unwrap()
                .get_values()
                .len(),
            5
        );
        assert!(result
            .get_option("letters")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("a"));
        assert!(result
            .get_option("letters")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("b"));
        assert!(result
            .get_option("letters")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("c"));
        assert!(result
            .get_option("letters")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("d"));
        assert!(result
            .get_option("letters")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("e"));
        assert!(result
            .get_option("numbers")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("1"));
        assert!(result.arg().unwrap().contains("one"));
        assert!(result.arg().unwrap().contains("two"));
        assert!(result.arg().unwrap().contains("three"));

        // --numbers only accepts 2 arguments but was 3
        assert_eq!(
            parse_with(
                "--letters a b c d e --numbers 1 2 3 -- one two three",
                command.clone()
            )
            .err()
            .unwrap()
            .kind(),
            &ErrorKind::InvalidArgumentCount
        );
    }

    #[test]
    fn parse_result_error_kind_test() {
        let command = Command::new("MyApp")
            .arg(Argument::with_name("values").values_count(0..5))
            .subcommand(Command::new("version"))
            .option(
                CommandOption::new("range")
                    .alias("r")
                    .arg(Argument::with_name("min").validator(parse_validator::<i64>()))
                    .arg(Argument::with_name("max").validator(parse_validator::<i64>())),
            )
            .option(CommandOption::new("A").alias("a"))
            .option(CommandOption::new("B").alias("b"))
            .subcommand(
                Command::new("read").option(
                    CommandOption::new("mode")
                        .required(true)
                        .arg(Argument::with_name("mode").valid_values(&["low", "mid", "high"])),
                ),
            )
            .subcommand(
                Command::new("data")
                    .subcommand(Command::new("set").arg(Argument::with_name("value")))
                    .subcommand(Command::new("get")),
            );

        let err_kind = move |value: &str| -> ErrorKind {
            parse_with(value, command.clone())
                .err()
                .unwrap_or_else(|| panic!("{}", value))
                .kind()
                .clone()
        };

        assert!(matches!(err_kind("version 1 2 3"), ErrorKind::InvalidArgumentCount));
        assert!(matches!(err_kind("-- 1 2 3 4 5"), ErrorKind::InvalidArgumentCount));
        assert!(matches!(err_kind("--range 0"), ErrorKind::InvalidArgumentCount));
        assert!(matches!(err_kind("--range 1 2 3 -- "),ErrorKind::InvalidArgumentCount));
        assert!(matches!(err_kind("-r=0=1"), ErrorKind::InvalidExpression));
        assert!(matches!(err_kind("--range 10 b"), ErrorKind::InvalidArgument(x) if x == "b"));
        assert!(matches!(err_kind("--C"), ErrorKind::UnexpectedOption(o) if o == "--C"));
        assert!(matches!(err_kind("data write"), ErrorKind::UnexpectedCommand(x) if x == "write"));
        assert!(matches!(err_kind("read"), ErrorKind::MissingOption(x) if x == "mode"));
        assert!(matches!(err_kind("read --mode lo"), ErrorKind::InvalidArgument(x) if x == "lo"));
        assert!(matches!(err_kind("read --mode low mid"),ErrorKind::InvalidArgumentCount));
        assert!(matches!(err_kind("data clear"), ErrorKind::UnexpectedCommand(x) if x == "clear"));
        assert!(matches!(err_kind("data get 0"), ErrorKind::InvalidArgumentCount));
        assert!(matches!(err_kind("data set \"Hello World\" Bye"),ErrorKind::InvalidArgumentCount));
    }

    #[test]
    fn parse_result_option_bool_flag_test() {
        let command = Command::new("MyApp").option(
            CommandOption::new("enable").arg(
                Argument::with_name("enable")
                    .values_count(0..=1)
                    .validator(parse_validator::<bool>()),
            ),
        );

        let res1 = parse_with("--enable true", command.clone()).unwrap();
        assert_eq!(
            res1.get_option("enable")
                .unwrap()
                .get_arg()
                .unwrap()
                .get_values()[0],
            "true".to_owned()
        );

        let res2 = parse_with("--enable false", command.clone()).unwrap();
        assert_eq!(
            res2.get_option("enable")
                .unwrap()
                .get_arg()
                .unwrap()
                .get_values()[0],
            "false".to_owned()
        );

        let res3 = parse_with("--enable", command.clone()).unwrap();
        assert!(res3.contains_option("enable"));

        let res4 = parse_with("", command.clone()).unwrap();
        assert!(!res4.contains_option("enable"));
    }

    #[test]
    fn parse_result_arg_default_values_test1(){
        let command = Command::new("MyApp")
            .arg(Argument::with_name("min").default(1))
            .arg(Argument::with_name("max"));

        let result1 = parse_with("10", command.clone()).unwrap();
        assert!(result1.args.get("min").unwrap().contains("1"));
        assert!(result1.args.get("max").unwrap().contains("10"));

        let result2 = parse_with("5 12", command.clone()).unwrap();
        assert!(result2.args.get("min").unwrap().contains("5"));
        assert!(result2.args.get("max").unwrap().contains("12"));
    }

    #[test]
    #[should_panic]
    fn parse_result_arg_default_values_test2(){
        let _command = Command::new("MyApp")
            .arg(Argument::with_name("min").default(1))
            .arg(Argument::with_name("max").default(10));
    }

    #[test]
    fn parse_result_option_default_values_test1(){
        let command = Command::new("MyApp")
            .option(CommandOption::new("range")
                .arg(Argument::with_name("start").default(1))
                .arg(Argument::with_name("end")));

        let result1 = parse_with("--range 22", command.clone()).unwrap();
        assert!(result1.get_option_args("range")
            .unwrap()
            .get("start")
            .unwrap()
            .contains("1"));
        assert!(result1.get_option_args("range")
            .unwrap()
            .get("end")
            .unwrap()
            .contains("22"));

        let result2 = parse_with("--range 10 25", command.clone()).unwrap();
        assert!(result2.get_option_args("range")
            .unwrap()
            .get("start")
            .unwrap()
            .contains("10"));
        assert!(result2.get_option_args("range")
            .unwrap()
            .get("end")
            .unwrap()
            .contains("25"));
    }

    #[test]
    #[should_panic]
    fn parse_result_option_default_values_test2(){
        let _command = Command::new("MyApp")
            .option(CommandOption::new("range")
                .arg(Argument::with_name("start").default(1))
                .arg(Argument::with_name("end").default(20)));
    }

    #[test]
    fn parse_result_allow_multiple_test(){
        let command = Command::new("MyApp")
            .option(CommandOption::new("values")
                .multiple(true)
                .arg(Argument::one_or_more("values")));

        let result1 = parse_with("--values 5 6", command.clone()).unwrap();
        assert!(result1.get_option_arg("values").unwrap().contains("5"));
        assert!(result1.get_option_arg("values").unwrap().contains("6"));

        let result2 = parse_with("--values 1 2 --values 3 4", command.clone()).unwrap();
        assert!(result2.get_option_arg("values").unwrap().contains("1"));
        assert!(result2.get_option_arg("values").unwrap().contains("2"));
        assert!(result2.get_option_arg("values").unwrap().contains("3"));
        assert!(result2.get_option_arg("values").unwrap().contains("4"));
    }
}
