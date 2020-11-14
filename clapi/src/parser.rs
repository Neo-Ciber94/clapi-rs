use crate::context::Context;
use crate::error::{Error, ErrorKind, Result};
use crate::option::{Options, CommandOption};
use crate::parse_result::ParseResult;
use crate::tokenizer::{DefaultTokenizer, Token, Tokenizer};
use crate::utils::Then;
use std::borrow::Borrow;
use crate::command::Command;

/// A trait for parse command arguments.
pub trait Parser<Args> {
    /// Parse the provided command arguments and returns a `Ok(ParseResult)` if not error is found,
    /// otherwise returns `Err(Error)`.
    fn parse(&mut self, context: &Context, args: Args) -> Result<ParseResult>;
}

/// A default implementation of the `Parser` trait.
#[derive(Debug, Default)]
pub struct DefaultParser;
impl<'a, S, I> Parser<I> for DefaultParser
where
    S: Borrow<str> + 'a,
    I: IntoIterator<Item = S>,
{
    fn parse(&mut self, context: &Context, args: I) -> Result<ParseResult> {
        let mut tokenizer = DefaultTokenizer::default();
        let tokens = tokenizer.tokenize(context, args)?;

        let mut iterator = tokens.iter().peekable();
        let mut result_options = Options::new();
        let mut command = context.root().as_ref();

        // Finds the executing command
        while let Some(Token::Cmd(name)) = iterator.peek() {
            command = command.get_child(name.as_str()).ok_or_else(|| {
                Error::new_parse_error(
                    ErrorKind::UnrecognizedCommand(name.clone()),
                    command.clone(),
                    None,
                    None,
                )
            })?;

            iterator.next();
        }

        // Gets the commands options
        while let Some(Token::Opt(prefix, s)) = iterator.peek() {
            if let Some(mut option) = get_option_prefixed(context, command, prefix, s).cloned() {
                // Consumes token
                iterator.next();

                // If the option take args, add them
                if option.args().take_args() {
                    let mut option_args = Vec::new();
                    let max_arg_count = option.args().arity().max_arg_count();

                    while let Some(t) = iterator.peek() {
                        // If the option don't takes more arguments exit
                        if option_args.len() >= max_arg_count {
                            break;
                        }

                        if let Token::Arg(s) = t {
                            option_args.push(s);
                            iterator.next();
                        } else {
                            break;
                        }
                    }

                    option.set_args_values(option_args.clone()).or_else(|e| {
                        let args = option_args.iter().map(|s| (*s).clone()).collect();

                        Err(Error::new_parse_error(
                            e.kind().clone(),
                            command.clone(),
                            Some(option.clone()),
                            Some(args),
                        ))
                    })?;
                }

                result_options.add(option);
            } else {
                return Err(Error::new_parse_error(
                    ErrorKind::UnrecognizedOption(s.clone()),
                    command.clone(),
                    None,
                    None,
                ));
            }
        }

        // If the current is `end of options` skip it
        if let Some(Token::EOO) = iterator.peek() {
            iterator.next();
        }

        // Check required options
        let required_options = command
            .then(|c| c.options().iter())
            .filter(|o| o.is_required());

        for opt in required_options {
            if !result_options.contains(opt.name()) {
                return Err(Error::from(ErrorKind::MissingOption(opt.name().to_owned())));
            }
        }

        // Adds options with default values
        let default_options = command
            .then(|c| c.options().iter())
            .filter(|o| o.args().has_default_values());

        for opt in default_options {
            if !result_options.contains(opt.name()) {
                result_options.add(opt.clone());
            }
        }

        // Sets the rest of the args to the command
        let mut rest_args = iterator
            .map(|t| t.clone().into_string())
            .collect::<Vec<String>>();

        // Sets default values if there is not args
        if rest_args.is_empty() && command.args().has_default_values() {
            for arg in command.args().default_values() {
                rest_args.push(arg.clone());
            }
        }

        // Clones the command and set the options and args
        let mut result_command = command.clone().set_new_options(result_options);

        result_command
            .set_args_values(rest_args.as_slice())
            .or_else(|e| {
                Err(Error::new_parse_error(
                    e.kind().clone(),
                    result_command.clone(),
                    None,
                    Some(rest_args),
                ))
            })?;

        Ok(ParseResult::new(result_command))
    }
}

fn get_option_prefixed<'a>(context: &'a Context, command: &'a Command, prefix: &'a str, option: &'a str) -> Option<&'a CommandOption>{
    if context.is_name_prefix(prefix) {
        return command.options().get_by_name(option);
    }

    if context.is_alias_prefix(prefix) {
        return command.options().get_by_alias(option);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arg_count::ArgCount;
    use crate::args::Arguments;
    use crate::command::Command;
    use crate::command_line::into_arg_iterator;
    use crate::option::CommandOption;
    use crate::root_command::RootCommand;

    fn parse(value: &str) -> Result<ParseResult> {
        let root =
            RootCommand::new()
                .set_option(CommandOption::new("version").set_alias("v"))
                .set_option(CommandOption::new("author").set_alias("a"))
                .set_command(Command::new("echo").set_args(Arguments::new(1..)))
                .set_command(
                    Command::new("pick")
                        .set_args(Arguments::new(ArgCount::new(1, 2)))
                        .set_option(CommandOption::new("color").set_args(
                            Arguments::new(1).set_valid_values(&["red", "blue", "green"]),
                        )),
                )
                .set_command(
                    Command::new("any").set_option(
                        CommandOption::new("numbers")
                            .set_required(true)
                            .set_args(Arguments::new(1..)),
                    ),
                );

        let values = into_arg_iterator(value);
        let context = Context::new(root);
        let mut parser = DefaultParser::default();
        parser.parse(&context, values)
    }

    #[test]
    fn parse_test1() {
        let result1 = parse("version");
        assert!(result1.is_err());

        let result2 = parse("--version").unwrap();
        assert!(result2.options().contains("version"));
    }

    #[test]
    fn parse_test2() {
        let result1 = parse("version");
        assert!(result1.is_err());

        let result2 = parse("--version").unwrap();
        assert!(result2.options().contains("version"));
        assert!(result2.options().contains("v"));
    }

    #[test]
    fn parse_test3() {
        let result1 = parse("--version author");
        assert!(result1.is_err());

        let result2 = parse("--version --author").unwrap();
        println!("{:#?}", result2.options());
        assert!(result2.options().contains("version"));
        assert!(result2.options().contains("v"));
        assert!(result2.options().contains("author"));
        assert!(result2.options().contains("a"));
    }

    #[test]
    fn parse_test4() {
        let result = parse("any --numbers=1,2,3,4").unwrap();
        assert_eq!(result.command().name(), "any");
        assert_eq!(
            result.options().get("numbers"),
            Some(&CommandOption::new("numbers"))
        );

        let args = result.options().get_args("numbers").unwrap();
        assert!(args.contains("1"));
        assert!(args.contains("2"));
        assert!(args.contains("3"));
        assert!(args.contains("4"));
    }

    #[test]
    fn parse_test5() {
        let result = parse("any --numbers:1,2,3,4").unwrap();
        assert_eq!(result.command().name(), "any");
        assert_eq!(
            result.options().get("numbers"),
            Some(&CommandOption::new("numbers"))
        );

        let args = result.options().get_args("numbers").unwrap();
        assert!(args.contains("1"));
        assert!(args.contains("2"));
        assert!(args.contains("3"));
        assert!(args.contains("4"));
    }

    #[test]
    fn parse_test6() {
        let result = parse("any --numbers=1,2,3 -- 4");
        assert!(result.is_err())
    }

    #[test]
    fn parse_test7() {
        let result = parse("any --numbers=1,2,3, 4");
        assert!(result.is_ok());

        let result = result.unwrap();
        let options = result.options();
        let args = options.get_args("numbers").unwrap();
        assert!(args.contains("1"));
        assert!(args.contains("2"));
        assert!(args.contains("3"));
        assert!(args.contains("4"));
    }

    #[test]
    fn parser_ok_test() {
        assert!(parse(" ").is_ok());
        assert!(parse("").is_ok());
        assert!(parse("--").is_ok());
        assert!(parse("--version").is_ok());
        assert!(parse("--author").is_ok());
        assert!(parse("-v").is_ok());
        assert!(parse("-a").is_ok());
    }

    #[test]
    fn parse_error_test() {
        assert!(parse("create").is_err());
        assert!(parse("create --path=hello.txt").is_err());
        assert!(parse("any --numbers").is_err());
        assert!(parse("any 5").is_err());
        assert!(parse("-version").is_err());
        assert!(parse("-author").is_err());
        assert!(parse("--v").is_err());
        assert!(parse("--a").is_err());
    }
}
