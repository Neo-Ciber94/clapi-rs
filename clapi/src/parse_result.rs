use crate::args::ArgumentList;
use crate::command::Command;
use crate::option::OptionList;
use crate::Argument;
use std::slice::Iter;

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

    // Returns the executing command.
    #[doc(hidden)]
    pub fn executing_command(&self) -> &Command {
        &self.command
    }

    /// Returns the name of the executing command.
    pub fn command_name(&self) -> &str {
        self.command.get_name()
    }

    /// Returns the version of the executing command or `None`.
    pub fn command_version(&self) -> Option<&str> {
        self.command.get_version()
    }

    /// Returns the help message of the executing command or `None`.
    pub fn command_help(&self) -> Option<&str> {
        self.command.get_help()
    }

    /// Returns the usage message of the executing command or `None`.
    pub fn command_usage(&self) -> Option<&str> {
        self.command.get_usage()
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

    /// Gets the value of the argument with the given name.
    pub fn value_of(&self, arg_name: &str) -> Option<&str> {
        self.args
            .get(arg_name)
            .map(|arg| arg.get_values())
            .filter(|values| values.len() == 1)
            .map(|values| values[0].as_str())
    }

    /// Gets an iterator over the values of the argument with the given name.
    pub fn values_of(&self, arg_name: &str) -> Option<Values<'_>> {
        if let Some(arg) = self.args.get(arg_name) {
            Some(Values {
                values: arg.get_values(),
            })
        } else {
            None
        }
    }

    /// Gets the value of the argument of the given option.
    pub fn value_of_option(&self, option_name: &str) -> Option<&str> {
        self.options
            .get(option_name)
            .map(|opt| opt.get_arg())
            .flatten()
            .map(|arg| arg.get_values())
            .filter(|values| values.len() == 1)
            .map(|values| values[0].as_str())
    }

    /// Gets an iterator over the values of the arguments of the given option.
    pub fn values_of_option(&self, option_name: &str) -> Option<Values<'_>> {
        if let Some(option) = self.options.get(option_name) {
            let arg = option.get_arg()?;
            Some(Values {
                values: arg.get_values(),
            })
        } else {
            None
        }
    }
}

/// An iterator over the values of an argument or option.
#[derive(Debug, Clone)]
pub struct Values<'a> {
    values: &'a [String],
}

impl<'a> Values<'a> {
    /// Returns an iterator over the argument valeus.
    pub fn iter(&self) -> Iter<'_, String> {
        self.values.iter()
    }

    /// Returns the number of values.
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns `true` if contains the value.
    #[inline]
    pub fn contains<S: AsRef<str>>(&self, value: S) -> bool {
        self.values.iter().any(|s| s == value.as_ref())
    }

    /// Returns an slice to the inner values.
    #[inline]
    pub fn inner(&self) -> &'a [String] {
        self.values
    }
}

impl<'a> IntoIterator for Values<'a> {
    type Item = &'a String;
    type IntoIter = Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a> IntoIterator for &'a Values<'a> {
    type Item = &'a String;
    type IntoIter = Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::validate_type;
    use crate::{split_into_args, CommandOption, Context, ErrorKind, Parser};

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
                        .validator(validate_type::<u64>())
                        .default(1),
                ),
            )
            .option(
                CommandOption::new("color")
                    .alias("c")
                    .arg(Argument::with_name("color").valid_values(&["red", "blue", "green"])),
            );

        let result = parse_with("--repeat 2 -c red hello world!", command.clone()).unwrap();
        assert!(result.options().contains("repeat"));
        assert!(result.options().contains("r"));
        assert!(result.options().contains("color"));
        assert!(result.options().contains("c"));
        assert_eq!(
            result
                .options()
                .get("repeat")
                .unwrap()
                .get_arg()
                .unwrap()
                .convert::<u64>()
                .ok(),
            Some(2)
        );
        assert!(result
            .options()
            .get("color")
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
            .options()
            .get("repeat")
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
                    .arg(Argument::with_name("times").validator(validate_type::<u64>())),
            )
            .arg(Argument::zero_or_more("values"));

        let result = parse_with("--times 1 -- one two three", command.clone()).unwrap();
        assert!(result.options().contains("times"));
        assert!(result
            .options()
            .get("times")
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
                        .validator(validate_type::<bool>())
                        .values_count(0..=1),
                ),
            );

        let result = parse_with("--hour -m -s --enable false", command.clone()).unwrap();
        assert_eq!(result.args().len(), 0);
        assert!(result.options().contains("hour"));
        assert!(result.options().contains("minute"));
        assert!(result.options().contains("second"));
        assert!(result
            .options()
            .get("enable")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("false"));
    }

    #[test]
    fn parse_result_multiple_args_test() {
        let command = Command::new("MyApp")
            .arg(Argument::with_name("min").validator(validate_type::<i64>()))
            .arg(Argument::with_name("max").validator(validate_type::<i64>()))
            .option(
                CommandOption::new("replace")
                    .alias("r")
                    .arg(Argument::with_name("from"))
                    .arg(Argument::with_name("to")),
            );

        let result = parse_with("--replace a A -- 2 10", command.clone()).unwrap();
        assert!(result
            .options()
            .get("replace")
            .unwrap()
            .get_args()
            .get("from")
            .unwrap()
            .contains("a"));
        assert!(result
            .options()
            .get("replace")
            .unwrap()
            .get_args()
            .get("to")
            .unwrap()
            .contains("A"));
        assert_eq!(
            result.args().get("min").unwrap().convert::<i64>().ok(),
            Some(2)
        );
        assert_eq!(
            result.args().get("max").unwrap().convert::<i64>().ok(),
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
            .option(
                CommandOption::new("D")
                    .alias("d")
                    .arg(Argument::one_or_more("d")),
            );

        let result1 = parse_with("--A --B -- --C", command.clone()).unwrap();
        assert_eq!(result1.options().len(), 2);
        assert_eq!(result1.arg().unwrap().get_values().len(), 1);
        assert!(result1.options().contains("A"));
        assert!(result1.options().contains("B"));
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
        assert!(result4.options().get_arg("D").unwrap().contains("1"));
        assert!(result4.options().get_arg("D").unwrap().contains("2"));
        assert!(result4.options().get_arg("D").unwrap().contains("3"));
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
                        .validator(validate_type::<char>()),
                ),
            )
            .option(
                CommandOption::new("numbers").arg(
                    Argument::with_name("numbers")
                        .values_count(1..=2)
                        .validator(validate_type::<i64>()),
                ),
            );

        let result = parse_with(
            "--letters a b c d e --numbers 1 -- one two three",
            command.clone(),
        )
        .unwrap();

        assert_eq!(
            result
                .options()
                .get("letters")
                .unwrap()
                .get_arg()
                .unwrap()
                .get_values()
                .len(),
            5
        );
        assert!(result
            .options()
            .get("letters")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("a"));
        assert!(result
            .options()
            .get("letters")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("b"));
        assert!(result
            .options()
            .get("letters")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("c"));
        assert!(result
            .options()
            .get("letters")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("d"));
        assert!(result
            .options()
            .get("letters")
            .unwrap()
            .get_arg()
            .unwrap()
            .contains("e"));
        assert!(result
            .options()
            .get("numbers")
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
                    .arg(Argument::with_name("min").validator(validate_type::<i64>()))
                    .arg(Argument::with_name("max").validator(validate_type::<i64>())),
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

        assert!(matches!(
            err_kind("version 1 2 3"),
            ErrorKind::InvalidArgumentCount
        ));
        assert!(matches!(
            err_kind("-- 1 2 3 4 5"),
            ErrorKind::InvalidArgumentCount
        ));
        assert!(matches!(
            err_kind("--range 0"),
            ErrorKind::InvalidArgumentCount
        ));
        assert!(matches!(
            err_kind("--range 1 2 3 -- "),
            ErrorKind::InvalidArgumentCount
        ));
        assert!(matches!(err_kind("-r=0=1"), ErrorKind::InvalidExpression));
        assert!(
            matches!(err_kind("--range 10 b"), ErrorKind::InvalidArgument(arg) if arg == "max")
        );
        assert!(matches!(err_kind("--C"), ErrorKind::UnexpectedOption(o) if o == "--C"));
        assert!(matches!(err_kind("data write"), ErrorKind::UnexpectedCommand(x) if x == "write"));
        assert!(matches!(err_kind("read"), ErrorKind::MissingOption(x) if x == "mode"));
        assert!(
            matches!(err_kind("read --mode lo"), ErrorKind::InvalidArgument(arg) if arg == "mode")
        );
        assert!(matches!(
            err_kind("read --mode low mid"),
            ErrorKind::InvalidArgumentCount
        ));
        assert!(matches!(err_kind("data clear"), ErrorKind::UnexpectedCommand(x) if x == "clear"));
        assert!(matches!(
            err_kind("data get 0"),
            ErrorKind::InvalidArgumentCount
        ));
        assert!(matches!(
            err_kind("data set \"Hello World\" Bye"),
            ErrorKind::InvalidArgumentCount
        ));
    }

    #[test]
    fn parse_result_option_bool_flag_test() {
        let command = Command::new("MyApp").option(
            CommandOption::new("enable").arg(
                Argument::with_name("enable")
                    .values_count(0..=1)
                    .validator(validate_type::<bool>()),
            ),
        );

        let res1 = parse_with("--enable true", command.clone()).unwrap();
        assert_eq!(
            res1.options()
                .get("enable")
                .unwrap()
                .get_arg()
                .unwrap()
                .get_values()[0],
            "true".to_owned()
        );

        let res2 = parse_with("--enable false", command.clone()).unwrap();
        assert_eq!(
            res2.options()
                .get("enable")
                .unwrap()
                .get_arg()
                .unwrap()
                .get_values()[0],
            "false".to_owned()
        );

        let res3 = parse_with("--enable", command.clone()).unwrap();
        assert!(res3.options().contains("enable"));

        let res4 = parse_with("", command.clone()).unwrap();
        assert!(!res4.options().contains("enable"));
    }

    #[test]
    fn parse_result_arg_default_values_test1() {
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
    fn parse_result_arg_default_values_test2() {
        let _command = Command::new("MyApp")
            .arg(Argument::with_name("min").default(1))
            .arg(Argument::with_name("max").default(10));
    }

    #[test]
    fn parse_result_option_default_values_test1() {
        let command = Command::new("MyApp").option(
            CommandOption::new("range")
                .arg(Argument::with_name("start").default(1))
                .arg(Argument::with_name("end")),
        );

        let result1 = parse_with("--range 22", command.clone()).unwrap();
        assert!(result1
            .options()
            .get_args("range")
            .unwrap()
            .get("start")
            .unwrap()
            .contains("1"));
        assert!(result1
            .options()
            .get_args("range")
            .unwrap()
            .get("end")
            .unwrap()
            .contains("22"));

        let result2 = parse_with("--range 10 25", command.clone()).unwrap();
        assert!(result2
            .options()
            .get_args("range")
            .unwrap()
            .get("start")
            .unwrap()
            .contains("10"));
        assert!(result2
            .options()
            .get_args("range")
            .unwrap()
            .get("end")
            .unwrap()
            .contains("25"));
    }

    #[test]
    #[should_panic]
    fn parse_result_option_default_values_test2() {
        let _command = Command::new("MyApp").option(
            CommandOption::new("range")
                .arg(Argument::with_name("start").default(1))
                .arg(Argument::with_name("end").default(20)),
        );
    }

    #[test]
    fn parse_result_allow_multiple_test() {
        let command = Command::new("MyApp").option(
            CommandOption::new("values")
                .multiple(true)
                .arg(Argument::one_or_more("values")),
        );

        let result1 = parse_with("--values 5 6", command.clone()).unwrap();
        assert!(result1.options().get_arg("values").unwrap().contains("5"));
        assert!(result1.options().get_arg("values").unwrap().contains("6"));

        let result2 = parse_with("--values 1 2 --values 3 4", command.clone()).unwrap();
        assert!(result2.options().get_arg("values").unwrap().contains("1"));
        assert!(result2.options().get_arg("values").unwrap().contains("2"));
        assert!(result2.options().get_arg("values").unwrap().contains("3"));
        assert!(result2.options().get_arg("values").unwrap().contains("4"));
    }

    #[test]
    fn parse_global_option_test() {
        let command = Command::new("MyApp")
            .option(
                CommandOption::new("color")
                    .global(true)
                    .arg(Argument::new().valid_values(vec!["red", "green", "blue"])),
            )
            .subcommand(Command::new("echo").arg(Argument::one_or_more("values")));

        let result = parse_with("echo --color red -- hello world", command.clone()).unwrap();
        assert_eq!(result.command_name(), "echo");
        assert!(result.options().get_arg("color").unwrap().contains("red"));
        assert!(result.args().get("values").unwrap().contains("hello"));
        assert!(result.args().get("values").unwrap().contains("world"));
    }

    #[test]
    fn parse_required_global_option_test() {
        let command = Command::new("MyApp")
            .option(CommandOption::new("flag").required(true).global(true))
            .subcommand(Command::new("echo").arg(Argument::one_or_more("values")));

        let result = parse_with("echo hello world", command.clone());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            &ErrorKind::MissingOption("flag".to_owned())
        );

        assert!(parse_with("echo --flag hello world", command.clone()).is_ok())
    }

    #[test]
    fn value_of_test() {
        let command = Command::new("MyApp").arg(Argument::with_name("color"));

        let result = parse_with("red", command.clone()).unwrap();
        assert_eq!("red", result.value_of("color").unwrap());
    }

    #[test]
    fn values_of_test() {
        let command = Command::new("MyApp").arg(Argument::one_or_more("colors"));

        let result = parse_with("red blue green", command.clone()).unwrap();
        assert_eq!(
            vec!["red".to_owned(), "blue".to_owned(), "green".to_owned()],
            result
                .values_of("colors")
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn value_of_option_test() {
        let command = Command::new("MyApp")
            .option(CommandOption::new("size").arg(Argument::with_name("size")));

        let result = parse_with("--size sm", command.clone()).unwrap();
        assert_eq!("sm", result.value_of_option("size").unwrap());
    }

    #[test]
    fn values_of_option_test() {
        let command = Command::new("MyApp")
            .option(CommandOption::new("sizes").arg(Argument::one_or_more("sizes")));

        let result = parse_with("--sizes sm md lg", command.clone()).unwrap();
        assert_eq!(
            vec!["sm".to_owned(), "md".to_owned(), "lg".to_owned()],
            result
                .values_of_option("sizes")
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<String>>()
        );
    }
}
