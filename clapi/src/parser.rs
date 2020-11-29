use crate::context::Context;
use crate::error::{Error, ErrorKind, Result};
use crate::option::{OptionList, CommandOption};
use crate::parse_result::ParseResult;
use crate::tokenizer::{DefaultTokenizer, Token, Tokenizer};
use crate::utils::Then;
use std::borrow::Borrow;
use crate::command::Command;
use crate::args::ArgumentList;

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
        let mut command_options = OptionList::new();
        let mut command = context.root();

        // Finds the executing command
        while let Some(Token::Cmd(name)) = iterator.peek() {
            command = command.find_subcommand(name.as_str()).ok_or_else(|| {
                Error::new_parse_error(
                    Error::from(ErrorKind::UnrecognizedCommand(name.clone())),
                    command.clone(),
                    None,
                    None,
                )
            })?;

            iterator.next();
        }

        // Gets the commands options
        while let Some(Token::Opt(prefix, s)) = iterator.peek() {
            if let Some(option) = get_option_prefixed(context, command, prefix, s) {
                // Consumes option token
                iterator.next();

                if option.take_args() {
                    let mut option_args = ArgumentList::new();
                    let mut option_args_iter = option.get_args()
                        .iter()
                        .peekable();

                    while let Some(arg) = option_args_iter.next(){
                        let mut values = Vec::new();
                        let max_count = arg.get_arg_count().max();
                        let mut count = 0;

                        while count < max_count {
                            if let Some(Token::Arg(value)) = iterator.peek() {
                                iterator.next();
                                values.push(value.clone());
                                count += 1;
                            } else {
                                break;
                            }
                        }

                        // If there is no more args, check if there is an `end of arguments`
                        if option_args_iter.peek().is_none() {
                            if iterator.peek().map_or(false, |t| !t.is_option()){
                                // https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html
                                // Check Guide 10
                                // If there is an `--` (end of arguments) we pass all the values
                                // before to the last option (if any)
                                if let Some(mut index) = iterator.clone().position(|t| t.is_eoo()){
                                    while index > 0 {
                                        let t = iterator.next().unwrap().clone().into_string();
                                        values.push(t);
                                        index -= 1;
                                    }
                                }
                            }
                        }

                        // Sets the argument values
                        let mut arg = arg.clone();
                        arg.set_values(values)?;
                        option_args.add(arg);
                    }

                    // Sets the option arguments
                    command_options.add(option.clone().args(option_args));
                } else {
                    // Adds the option
                    command_options.add(option.clone());
                }
            } else {
                return Err(Error::new_parse_error(
                    Error::from(ErrorKind::UnrecognizedOption(prefix.clone(), s.clone())),
                    command.clone(),
                    None,
                    None,
                ));
            }
        }

        // We check for `end of arguments` if any we skip it if there is no arguments before it
        if let Some(index) = iterator.clone().position(|t| t.is_eoo()){
            // If there is arguments before `--` (end of arguments)
            // values are being passed to the last option which not exist.
            if index > 0 {
                // https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html
                // Check Guide 10

                // We get the last argument to provide a hint of the error
                let value = iterator.next().cloned().unwrap().into_string();
                return Err(
                    Error::new(
                        ErrorKind::InvalidArgument(value),
                        "there is no options that expect arguments"
                    )
                );
            } else {
                iterator.next();
            }
        }

        // Check required options
        let required_options = command
            .then(|c| c.get_options().iter())
            .filter(|o| o.is_required());

        for opt in required_options {
            if !command_options.contains(opt.get_name()) {
                return Err(Error::from(ErrorKind::MissingOption(opt.get_name().to_owned())));
            }
        }

        // Gets the options that takes default arguments
        let default_options = command
            .then(|c| c.get_options().iter())
            .filter(|o| o.get_args().iter().any(|a| a.has_default_values()));

        // Sets the options that takes default arguments
        for opt in default_options {
            if !command_options.contains(opt.get_name()) {
                command_options.add(opt.clone());
            }
        }

        let mut command_args = ArgumentList::new();
        let mut args_iter = command.get_args()
            .iter()
            .cloned()
            .peekable();

        while let Some(mut arg) = args_iter.next(){
            let mut values = Vec::new();

            if args_iter.peek().is_some() {
                let max_count = arg.get_arg_count().max();
                let mut count = 0;

                while count < max_count {
                    if let Some(Token::Arg(value)) = iterator.peek() {
                        iterator.next();
                        values.push(value.clone());
                        count += 1;
                    } else {
                        break;
                    }
                }
            } else {
                // If there is no `Argument`s left, pass the rest of the tokens as values
                while let Some(s) = iterator.next().cloned(){
                    values.push(s.into_string());
                }
            }

            // Sets the argument values
            // We attempt to set them even if the values is empty
            // to return an `invalid argument count` error.
            if values.len() > 0 || (values.is_empty() && !arg.has_default_values()) {
                arg.set_values(values)
                    .or_else(|error| {
                        Err(Error::new_parse_error(
                            error,
                            command.clone(),
                            None,
                            Some(command_args.clone()),
                        ))
                    })?;
            }

            command_args.add(arg);
        }

        // If there is more values which weren't consume, so the current command takes not args
        if iterator.peek().is_some() {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                format!("`{}` takes no arguments", command.get_name())
            ));
        }

        // Sets the command options and arguments
        let command = command.clone()
            .options(command_options)
            .args(command_args);

        Ok(ParseResult::new(command))
    }
}

fn get_option_prefixed<'a>(context: &'a Context, command: &'a Command, prefix: &'a str, option: &'a str) -> Option<&'a CommandOption>{
    if context.is_name_prefix(prefix) {
        return command.get_options().get_by_name(option);
    }

    if context.is_alias_prefix(prefix) {
        return command.get_options().get_by_alias(option);
    }

    None
}
