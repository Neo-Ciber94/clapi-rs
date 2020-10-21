use crate::args::{Arguments, Iter};
use crate::command::Command;
use crate::error::Result;
use crate::option::{CommandOption, Options};
use std::fmt::Display;
use std::str::FromStr;

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
    pub fn options(&self) -> &Options {
        &self.command.options()
    }

    /// Returns the arguments of the command.
    pub fn args(&self) -> &Arguments {
        self.command.args()
    }

    /// Returns `true` if the resulting command contains the option with the given name or alias.
    pub fn contains_option(&self, name_or_alias: &str) -> bool {
        self.command.options().contains(name_or_alias)
    }

    /// Returns `true` if the resulting command contains the given argument.
    pub fn contains_arg(&self, value: &str) -> bool {
        self.command.args().contains(value)
    }

    /// Returns the `CommandOption` with the given name or alias, or `None`.
    pub fn get_option(&self, name_or_alias: &str) -> Option<&CommandOption> {
        self.command.options().get(name_or_alias)
    }

    /// Returns the arguments of the option with the given name or alias, or `None` if not found.
    pub fn get_option_args(&self, name_or_alias: &str) -> Option<&[String]> {
        self.get_option(name_or_alias)
            .map(|o| o.args().values())
            .filter(|a| a.len() > 0)
    }

    /// Converts the first option argument value into the specified type, returns `None`
    /// if the option is not found.
    ///
    /// # Error
    /// - If there is no values to convert.
    /// - If this takes not args.
    /// - The value cannot be converted to type `T`.
    pub fn get_option_arg_as<T>(&self, name_or_alias: &str) -> Option<Result<T>>
    where
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        self.get_option(name_or_alias).map(|o| o.arg_as())
    }

    /// Returns an iterator that converts the argument values into the specified type,
    /// returns `None` if the option is not found.
    ///
    /// # Error
    /// - If there is no values to convert.
    /// - If this takes not args.
    /// - One of the values cannot be converted to type `T`.
    pub fn get_option_args_as<T>(&self, name_or_alias: &str) -> Option<Result<Iter<'_, T>>>
        where
            T: FromStr,
            <T as FromStr>::Err: Display,
    {
        self.get_option(name_or_alias).map(|o| o.args_as())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::validator::validator_for;
    use crate::args::Arguments;
    use crate::command_line::into_arg_iterator;
    use crate::context::Context;
    use crate::error::Result;
    use crate::parser::{DefaultParser, Parser};
    use crate::root_command::RootCommand;

    fn parse(value: &str) -> Result<ParseResult> {
        let root = RootCommand::new()
            .set_args(Arguments::new(0..=1))
            .set_option(
                CommandOption::new("number")
                    .set_alias("n")
                    .set_args(Arguments::new(1..).set_validator(validator_for::<i32>())),
            )
            .set_option(
                CommandOption::new("letter")
                    .set_alias("l")
                    .set_args(Arguments::new(1).set_validator(validator_for::<char>())),
            )
            .set_command(
                Command::new("select")
                    .set_args(Arguments::new(1..))
                    .set_option(CommandOption::new("sort")),
            )
            .set_command(
                Command::new("any")
                    .set_args(Arguments::new(1))
                    .set_option(CommandOption::new("A").set_alias("a"))
                    .set_option(CommandOption::new("B").set_alias("b"))
                    .set_option(CommandOption::new("C").set_alias("c")),
            );

        let context = Context::new(root);
        let mut parser = DefaultParser::default();
        parser.parse(&context, into_arg_iterator(value))
    }

    #[test]
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

    #[test]
    fn parse_result_test2() {
        let result = parse("select a z 1 9").unwrap();

        assert_eq!(result.command().name(), "select");
        assert!(result.contains_arg("a"));
        assert!(result.contains_arg("z"));
        assert!(result.contains_arg("1"));
        assert!(result.contains_arg("9"));
    }

    #[test]
    fn parse_result_test3() {
        let result = parse("select --sort 3 2 1").unwrap();

        assert_eq!(result.command().name(), "select");
        assert!(result.contains_option("sort"));
        assert!(result.contains_arg("3"));
        assert!(result.contains_arg("2"));
        assert!(result.contains_arg("1"));
    }

    #[test]
    fn parse_result_test4() {
        let result = parse("any --A --B --C hello").unwrap();

        assert_eq!(result.command().name(), "any");
        assert!(result.contains_arg("hello"));
        assert!(result.contains_option("A"));
        assert!(result.contains_option("B"));
        assert!(result.contains_option("C"));
    }

    #[test]
    fn parse_result_test5() {
        let result = parse("any --A --B --C -a -b -c hello").unwrap();

        assert_eq!(result.command().name(), "any");
        assert!(result.contains_arg("hello"));
        assert_eq!(result.options().len(), 3);
        assert!(result.contains_option("A"));
        assert!(result.contains_option("B"));
        assert!(result.contains_option("C"));
    }

    #[test]
    fn parse_result_error_test() {
        assert!(parse("-n").is_err());
        assert!(parse("-l").is_err());
        assert!(parse("select --sort").is_err());
        assert!(parse("any -a -b -c hello world").is_err());
    }

    #[test]
    fn parse_result_ok_test() {
        assert!(parse("-n 3 2 1").is_ok());
        assert!(parse("-l a").is_ok());
        assert!(parse("select --sort a b c").is_ok());
        assert!(parse("any -a -b -c \"hello world\"").is_ok());
    }
}
