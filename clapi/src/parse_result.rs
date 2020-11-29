use crate::command::Command;
use crate::option::{CommandOption, OptionList};
use crate::args::ArgumentList;
use crate::Argument;

/// Represents the result of a parse operation
/// and provides a set of methods to query over the values.
#[derive(Debug)]
pub struct ParseResult {
    command: Command,
}

impl ParseResult {
    /// Constructs a new `ParseResult`.
    pub fn new(command: Command) -> Self {
        ParseResult { command }
    }

    /// Returns the `Command` to execute.
    pub fn command(&self) -> &Command {
        &self.command
    }

    /// Returns the `Options` of the command.
    pub fn options(&self) -> &OptionList {
        &self.command.get_options()
    }

    /// Returns the argument of the command.
    pub fn arg(&self) -> Option<&Argument>{
        self.command.get_arg()
    }

    /// Returns the arguments of the command.
    pub fn args(&self) -> &ArgumentList {
        self.command.get_args()
    }

    /// Returns `true` if the resulting command contains the option with the given name or alias.
    pub fn contains_option<S: AsRef<str>>(&self, name_or_alias: S) -> bool {
        self.command.get_options().contains(name_or_alias)
    }

    /// Returns `true` if the resulting command contains an argument with the given name.
    pub fn contains_arg<S: AsRef<str>>(&self, arg_name: S) -> bool {
        self.command.get_args().contains(arg_name)
    }

    /// Returns the `CommandOption` with the given name or alias, or `None`.
    pub fn get_option<S: AsRef<str>>(&self, name_or_alias: S) -> Option<&CommandOption> {
        self.command.get_options().get(name_or_alias)
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
        self.get_option(name_or_alias)
            .map(|o| o.get_args())
    }

    /// Returns the `Argument` with the given name or `None` if not found.
    pub fn get_arg<S: AsRef<str>>(&self, arg_name: S) -> Option<&Argument>{
        self.command.get_args().get(arg_name)
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::{Context, DefaultParser, into_arg_iterator, Parser, ErrorKind};
    use crate::args::validator::parse_validator;

    fn parse_with(value: &str, command: Command) -> crate::Result<ParseResult> {
        let context = Context::new(command);
        let mut parser = DefaultParser::default();
        parser.parse(&context, into_arg_iterator(value))
    }

    #[test]
    fn parse_result_command_test(){
        let command = Command::new("My App")
            .arg(Argument::one_or_more("values"))
            .option(CommandOption::new("repeat")
                .alias("r")
                .arg(Argument::new("times")
                    .validator(parse_validator::<u64>())
                    .default(1)))
            .option(CommandOption::new("color")
                .alias("c")
                .arg(Argument::new("color")
                    .valid_values(&["red", "blue", "green"])));

        let result = parse_with("--repeat 2 -c red hello world!", command.clone()).unwrap();
        assert!(result.contains_option("repeat"));
        assert!(result.contains_option("r"));
        assert!(result.contains_option("color"));
        assert!(result.contains_option("c"));
        assert_eq!(result.get_option("repeat").unwrap().get_arg().unwrap().convert::<u64>().ok(), Some(2));
        assert!(result.get_option("color").unwrap().get_arg().unwrap().contains("red"));
        assert_eq!(result.arg().unwrap().get_values(), &["hello", "world!"]);

        // Whitespace test
        assert!(parse_with("\" \"", command.clone()).unwrap().arg().unwrap().contains(" "));

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
    fn parse_result_subcommand_test(){
        let command = Command::new("My App")
            .subcommand(Command::new("version"))
            .subcommand(Command::new("set")
                .arg(Argument::one_or_more("value"))
                .option(CommandOption::new("repeat")
                    .arg(Argument::new("count"))))
            .subcommand(Command::new("get"));

        let result = parse_with("set --repeat 1 1 2 3 4", command.clone()).unwrap();
        assert_eq!(result.command().get_name(), "set");
        assert!(result.get_option("repeat").unwrap().get_arg().unwrap().contains("1"));
        assert_eq!(result.arg().unwrap().get_values(), &["1".to_owned(), "2".to_owned(), "3".to_owned(), "4".to_owned()]);

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
    fn parse_result_required_option_test(){
        let command = Command::new("My App")
            .option(CommandOption::new("enable"))
            .option(CommandOption::new("times")
                .alias("t")
                .required(true)
                .arg(Argument::new("times")
                    .validator(parse_validator::<u64>())))
            .arg(Argument::zero_or_more("values"));

        let result = parse_with("--times 1 -- one two three", command.clone()).unwrap();
        assert!(result.contains_option("times"));
        assert!(result.get_option("times").unwrap().get_arg().unwrap().contains("1"));
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
    fn parse_result_options_test(){
        let command = Command::new("My App")
            .option(CommandOption::new("hour").alias("h"))
            .option(CommandOption::new("minute").alias("m"))
            .option(CommandOption::new("second").alias("s"))
            .option(CommandOption::new("enable")
                .arg(Argument::new("value")
                    .validator(parse_validator::<bool>())
                    .arg_count(0..=1)));

        let result = parse_with("--hour -m -s --enable false", command.clone()).unwrap();
        assert_eq!(result.args().len(), 0);
        assert!(result.contains_option("hour"));
        assert!(result.contains_option("minute"));
        assert!(result.contains_option("second"));
        assert!(result.get_option("enable").unwrap().get_arg().unwrap().contains("false"));
    }

    #[test]
    fn parse_result_multiple_args_test(){
        let command = Command::new("My App")
            .arg(Argument::new("min")
                .validator(parse_validator::<i64>()))
            .arg(Argument::new("max")
                .validator(parse_validator::<i64>()))
            .option(CommandOption::new("replace")
                .alias("r")
                .arg(Argument::new("from"))
                .arg(Argument::new("to")));

        let result = parse_with("--replace a A -- 2 10", command.clone()).unwrap();
        assert!(result.get_option("replace").unwrap().get_args().get("from").unwrap().contains("a"));
        assert!(result.get_option("replace").unwrap().get_args().get("to").unwrap().contains("A"));
        assert_eq!(result.get_arg("min").unwrap().convert::<i64>().ok(), Some(2));
        assert_eq!(result.get_arg("max").unwrap().convert::<i64>().ok(), Some(10));

        // Ok
        assert!(parse_with("2 10", command.clone()).is_ok());

        // Err
        assert!(parse_with("--replace hello HELLO", command.clone()).is_err());
        assert!(parse_with("25", command.clone()).is_err());
    }

    #[test]
    fn parse_result_eoa_test(){
        let command = Command::new("My App")
            .arg(Argument::one_or_more("args"))
            .option(CommandOption::new("A").alias("a"))
            .option(CommandOption::new("B").alias("b"))
            .option(CommandOption::new("C").alias("c"));

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
    }

    #[test]
    fn parse_result_variable_arg_count_test1(){
        let command = Command::new("My App")
            .arg(Argument::new("values").arg_count(0..=3))
            .option(CommandOption::new("letters")
                .arg(Argument::new("letters")
                    .arg_count(1..)
                    .validator(parse_validator::<char>())))
            .option(CommandOption::new("numbers")
                .arg(Argument::new("numbers")
                    .arg_count(1..=2)
                    .validator(parse_validator::<i64>())));

        let result = parse_with(
            "--letters a b c d e --numbers 1 -- one two three",
            command.clone()
        ).unwrap();

        assert_eq!(result.get_option("letters").unwrap().get_arg().unwrap().get_values().len(), 5);
        assert!(result.get_option("letters").unwrap().get_arg().unwrap().contains("a"));
        assert!(result.get_option("letters").unwrap().get_arg().unwrap().contains("b"));
        assert!(result.get_option("letters").unwrap().get_arg().unwrap().contains("c"));
        assert!(result.get_option("letters").unwrap().get_arg().unwrap().contains("d"));
        assert!(result.get_option("letters").unwrap().get_arg().unwrap().contains("e"));
        assert!(result.get_option("numbers").unwrap().get_arg().unwrap().contains("1"));
        assert!(result.arg().unwrap().contains("one"));
        assert!(result.arg().unwrap().contains("two"));
        assert!(result.arg().unwrap().contains("three"));

        // --numbers only accepts 2 arguments but was 3
        assert_eq!(
            parse_with(
                "--letters a b c d e --numbers 1 2 3 -- one two three",
                command.clone()).err().unwrap().kind(),
                &ErrorKind::InvalidArgumentCount);
    }

    #[test]
    fn parse_result_error_kind_test(){
        let command = Command::new("My App")
            .arg(Argument::new("values").arg_count(0..5))
            .subcommand(Command::new("version"))
            .option(CommandOption::new("range").alias("r")
                .arg(Argument::new("min").validator(parse_validator::<i64>()))
                .arg(Argument::new("max").validator(parse_validator::<i64>())))
            .option(CommandOption::new("A").alias("a"))
            .option(CommandOption::new("B").alias("b"))
            .subcommand(Command::new("read")
                .option(CommandOption::new("mode")
                    .required(true)
                    .arg(Argument::new("mode")
                        .valid_values(&["low", "mid", "high"]))))
            .subcommand(Command::new("data")
                .subcommand(Command::new("set")
                    .arg(Argument::new("value")))
                .subcommand(Command::new("get")));

        let err_kind = move |value: &str| -> ErrorKind {
            parse_with(value, command.clone()).err()
                .unwrap_or_else(|| panic!("{}", value))
                .kind()
                .clone()
        };

        assert!(matches!(err_kind("version 1 2 3"), ErrorKind::InvalidArgumentCount));
        assert!(matches!(err_kind("-- 1 2 3 4 5"), ErrorKind::InvalidArgumentCount));
        assert!(matches!(err_kind("--range 0"), ErrorKind::InvalidArgumentCount));
        assert!(matches!(err_kind("--range 1 2 3 -- "), ErrorKind::InvalidArgumentCount));
        assert!(matches!(err_kind("-r:0:1"), ErrorKind::InvalidExpression));
        assert!(matches!(err_kind("--range 10 b"), ErrorKind::InvalidArgument(x) if x == "b"));
        assert!(matches!(err_kind("--C"), ErrorKind::UnrecognizedOption(p, o) if p == "--" && o == "C"));
        assert!(matches!(err_kind("data write"), ErrorKind::UnrecognizedCommand(x) if x == "write"));
        assert!(matches!(err_kind("read"), ErrorKind::MissingOption(x) if x == "mode"));
        assert!(matches!(err_kind("read --mode lo"), ErrorKind::InvalidArgument(x) if x == "lo"));
        assert!(matches!(err_kind("read --mode low mid"), ErrorKind::InvalidArgumentCount));
        assert!(matches!(err_kind("data clear"), ErrorKind::UnrecognizedCommand(x) if x == "clear"));
        assert!(matches!(err_kind("data get 0"), ErrorKind::InvalidArgumentCount));
        assert!(matches!(err_kind("data set \"Hello World\" Bye"), ErrorKind::InvalidArgumentCount));
    }
}

//#[cfg(test)]
#[allow(dead_code)]
mod tests2 {
    use super::*;
    use crate::args::validator::parse_validator;
    use crate::command_line::into_arg_iterator;
    use crate::context::Context;
    use crate::error::Result;
    use crate::parser::{DefaultParser, Parser};
    use crate::args::Argument;

    fn parse(value: &str) -> Result<ParseResult> {
        let root = Command::root()
            .arg(Argument::new("args").arg_count(0..=2))
            .option(
                CommandOption::new("number")
                    .alias("n")
                    .arg(Argument::new("number").validator(parse_validator::<i32>())),
            )
            .option(
                CommandOption::new("letter")
                    .alias("l")
                    .arg(Argument::new("letter").validator(parse_validator::<char>())),
            )
            .subcommand(
                Command::new("select")
                    .arg(Argument::new("select"))
                    .option(CommandOption::new("sort")),
            )
            .subcommand(
                Command::new("any")
                    .arg(Argument::new("any"))
                    .option(CommandOption::new("A").alias("a"))
                    .option(CommandOption::new("B").alias("b"))
                    .option(CommandOption::new("C").alias("c")),
            );

        let context = Context::new(root);
        let mut parser = DefaultParser::default();
        parser.parse(&context, into_arg_iterator(value))
    }

    //#[test]
    fn parse_result_test1() {
        let result = parse("--number 1 2 3 --letter c -- hello").unwrap();

        assert!(result.contains_option("number"));
        assert!(result.contains_option("letter"));
        assert!(result.contains_arg("hello"));

        assert!(result
            .get_option_args("number")
            .unwrap()
            .contains(&String::from("1")));

        assert!(result
            .get_option_args("number")
            .unwrap()
            .contains(&String::from("2")));

        assert!(result
            .get_option_args("number")
            .unwrap()
            .contains(&String::from("3")));

        assert!(result
            .get_option_args("letter")
            .unwrap()
            .contains(&String::from("c")));
    }

    //#[test]
    fn parse_result_test2() {
        let result = parse("select a z 1 9").unwrap();

        assert_eq!(result.command().get_name(), "select");
        assert!(result.contains_arg("a"));
        assert!(result.contains_arg("z"));
        assert!(result.contains_arg("1"));
        assert!(result.contains_arg("9"));
    }

    //#[test]
    fn parse_result_test3() {
        let result = parse("select --sort 3 2 1").unwrap();

        assert_eq!(result.command().get_name(), "select");
        assert!(result.contains_option("sort"));
        assert!(result.contains_arg("3"));
        assert!(result.contains_arg("2"));
        assert!(result.contains_arg("1"));
    }

    //#[test]
    fn parse_result_test4() {
        let result = parse("any --A --B --C hello").unwrap();

        assert_eq!(result.command().get_name(), "any");
        assert!(result.contains_arg("hello"));
        assert!(result.contains_option("A"));
        assert!(result.contains_option("B"));
        assert!(result.contains_option("C"));
    }

    //#[test]
    fn parse_result_test5() {
        let result = parse("any --A --B --C -a -b -c hello").unwrap();

        assert_eq!(result.command().get_name(), "any");
        assert!(result.contains_arg("hello"));
        assert_eq!(result.options().len(), 3);
        assert!(result.contains_option("A"));
        assert!(result.contains_option("B"));
        assert!(result.contains_option("C"));
    }

    //#[test]
    fn parse_result_test6() {
        let result = parse("any --A --B -- --C").unwrap();

        assert_eq!(result.command().get_name(), "any");
        assert_eq!(result.options().len(), 2);
        assert!(result.contains_option("A"));
        assert!(result.contains_option("B"));
        assert_eq!(result.arg().unwrap().get_values().len(), 1);
        assert!(result.arg().unwrap().contains("--C"));
    }

    //#[test]
    fn parse_result_error_test() {
        assert!(parse("-n").is_err());
        assert!(parse("-l").is_err());
        assert!(parse("select --sort").is_err());
        assert!(parse("any -a -b -c hello world").is_err());
    }

    //#[test]
    fn parse_result_ok_test() {
        assert!(parse("-n 3 2 1").is_ok());
        assert!(parse("-l a").is_ok());
        assert!(parse("select --sort a b c").is_ok());
        assert!(parse("any -a -b -c \"hello world\"").is_ok());
        assert!(parse("--letter h --number 1 2 3 4 5").is_ok());
        assert!(parse("--letter h -- hello").is_ok());
        assert!(parse("--letter h hello --").is_ok());
    }
}
