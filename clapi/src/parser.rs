use std::borrow::Borrow;
use std::iter::Peekable;
use crate::args::ArgumentList;
use crate::command::Command;
use crate::context::Context;
use crate::error::{Error, ErrorKind, Result};
use crate::help::HelpKind;
use crate::option::{CommandOption, OptionList};
use crate::parse_result::ParseResult;
use crate::tokenizer::{Token, Tokenizer};
use crate::utils::Then;

/// A command-line argument parser.
#[derive(Debug)]
pub struct Parser;

impl Parser {
    pub fn parse<S, I>(&self, context: &Context, args: I, ) -> Result<ParseResult>
        where S: Borrow<str>,
              I: IntoIterator<Item = S> {
        let tokens = Tokenizer.tokenize(context, args)?;
        let mut iterator = tokens.iter().peekable();

        // Gets executing command
        let command = self.get_executing_command(context, &mut iterator)?;

        // Gets the commands options and its arguments
        let mut options = self.get_options(context, command, &mut iterator)?;

        // Skip next `end of arguments` token (if any)
        if let Some(index) = iterator.clone().position(|t| t.is_eoo()) {
            // If there is arguments before `--` (end of arguments)
            // values are being passed to the last option which not exist.
            //
            // For example: 1 2 3 -- Hello World
            // This is a error because there is no option to pass 1 2 3
            if index > 0 {
                // Check Guide 10
                // https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html

                // We get the last argument to provide a hint of the error
                let value = iterator.next().cloned().unwrap().into_string();
                return Err(Error::new(
                    ErrorKind::InvalidArgument(value),
                    "there is no options that expect arguments",
                ));
            } else {
                iterator.next();
            }
        }

        // Check and set required options (if any)
        self.set_required_options(command, &mut options)?;

        // Check and set options with default values (if any)
        self.set_default_options(command, &mut options)?;

        // Gets the command arguments
        let args = self.get_args(command, &options, &mut iterator)?;

        // If there is arguments left and the current command takes no arguments is an error
        if iterator.peek().is_some() {
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                format!("`{}` takes no arguments", command.get_name()),
            ));
        }

        // Sets the command, options and arguments
        Ok(ParseResult::new(command.clone(), options, args))
    }

    fn get_executing_command<'a, I>(&self, context: &'a Context, iterator: &mut Peekable<I>) -> Result<&'a Command> where I: Iterator<Item=&'a Token> {
        let mut command = context.root();
        while let Some(Token::Cmd(name)) = iterator.peek() {
            command = command.find_subcommand(name.as_str()).ok_or_else(|| {
                Error::new_parse_error(
                    Error::from(ErrorKind::UnrecognizedCommand(name.clone())),
                    ParseResult::new(
                        command.clone(),
                        OptionList::default(),
                        ArgumentList::default(),
                    ),
                )
            })?;

            iterator.next();
        }

        Ok(command)
    }

    fn get_options<'a, I>(&self, context: &Context, command: &Command, iterator: &mut Peekable<I>) -> Result<OptionList> where I: Iterator<Item=&'a Token> + Clone {
        let mut options = OptionList::new();

        while let Some(Token::Opt(prefix, s)) = iterator.peek() {
            if let Some(option) = get_option_prefixed(context, command, prefix, s) {
                // Consumes option token
                iterator.next();

                if option.take_args() {
                    let mut option_args = ArgumentList::new();
                    let mut option_args_iter = option.get_args().iter().cloned().peekable();
                    let require_default_values = self.require_default_values(option.get_args(), iterator);
                    let mut default_value_is_set = false;

                    while let Some(mut arg) = option_args_iter.next() {
                        // We take the first `Argument` that required a default values.
                        // Only 1 because multiple arguments with default values is no allowed.
                        if require_default_values && !default_value_is_set {
                            if arg.has_default_values() {
                                option_args.add(arg).unwrap();

                                // This is just a flag, `Argument`S with default values already have
                                // the default value set
                                default_value_is_set = true;
                                continue;
                            }
                        }

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

                        // If there is no more option args, check if there is an `end of arguments`
                        if option_args_iter.peek().is_none() {
                            if iterator.peek().map_or(false, |t| !t.is_option()) {
                                // Check Guide 10
                                // https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html
                                // If there is an `--` (end of arguments) we pass all the values
                                // before it to the last option as arguments (if any)
                                //
                                // Example: --numbers 1 2 3 -- hello world
                                // 1 2 3 are passed to the option `--numbers`
                                if let Some(mut index) = iterator.clone().position(|t| t.is_eoo()) {
                                    while index > 0 {
                                        let s = iterator.next().unwrap().clone().into_string();
                                        values.push(s);
                                        index -= 1;
                                    }
                                }
                            }
                        }

                        // Sets the argument values
                        arg.set_values(values).or_else(|error| {
                            // We add the last option to the error
                            let mut options = options.clone();
                            options.add(option.clone()).unwrap();

                            Err(Error::new_parse_error(
                                error,
                                ParseResult::new(
                                    command.clone(), options, ArgumentList::default()
                                ),
                            ))
                        })?;

                        option_args.add(arg).unwrap();
                    }

                    // Sets the option arguments
                    options
                        .add(option.args(option_args))
                        .unwrap();
                } else {
                    // Adds the option
                    options.add(option).unwrap();
                }
            } else {
                return Err(Error::new_parse_error(
                    Error::from(ErrorKind::UnrecognizedOption(prefix.clone(), s.clone())),
                    ParseResult::new(
                        command.clone(),
                        options.clone(),
                        ArgumentList::default(),
                    ),
                ));
            }
        }

        Ok(options)
    }

    fn get_args<'a, I>(&self, command: &Command, options: &OptionList, iterator: &mut Peekable<I>) -> Result<ArgumentList> where I: Iterator<Item=&'a Token> + Clone {
        let mut command_args = ArgumentList::new();
        let mut args_iter = command.get_args().iter().cloned().peekable();
        let require_default_values = self.require_default_values(command.get_args(), iterator);
        let mut default_value_is_set = false;

        while let Some(mut arg) = args_iter.next() {
            let mut values = Vec::new();

            // We take the first `Argument` that required a default values.
            // Only 1 because multiple arguments with default values is no allowed.
            if require_default_values && !default_value_is_set {
                if arg.has_default_values() {
                    command_args.add(arg).unwrap();

                    // This is just a flag, `Argument`S with default values already have
                    // the default value set
                    default_value_is_set = true;
                    continue;
                }
            }

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
                while let Some(t) = iterator.next().cloned() {
                    values.push(t.into_string());
                }
            }

            // Sets the argument values
            // We attempt to set the values even if empty to return `invalid argument count` error.
            if values.len() > 0 || (values.is_empty() && !arg.has_default_values()) {
                arg.set_values(values).or_else(|error| {
                    // We add the last arg
                    let mut args = command_args.clone();
                    args.add(arg.clone()).unwrap();

                    Err(Error::new_parse_error(
                        error,
                        ParseResult::new(
                            command.clone(), options.clone(), args
                        ),
                    ))
                })?;
            }

            command_args.add(arg).unwrap();
        }

        Ok(command_args)
    }

    fn set_required_options(&self, command: &Command, options: &mut OptionList) -> Result<()> {
        let required_options = command
            .then(|c| c.get_options().iter())
            .filter(|o| o.is_required());

        for opt in required_options {
            if !options.contains(opt.get_name()) {
                return Err(Error::from(ErrorKind::MissingOption(
                    opt.get_name().to_owned(),
                )));
            }
        }

        Ok(())
    }

    fn set_default_options(&self, command: &Command, options: &mut OptionList) -> Result<()> {
        let default_options = command
            .then(|c| c.get_options().iter())
            .filter(|o| o.get_args().iter().any(|a| a.has_default_values()));

        // Sets the options that takes default arguments
        for opt in default_options {
            if !options.contains(opt.get_name()) {
                options.add(opt.clone()).unwrap();
            }
        }

        Ok(())
    }

    /// Returns `true` if one of the arguments need to have a default value.
    ///
    /// This is true when there is no enough values for the arguments.
    /// For example: the arguments `min` (default 0) and `max` are declared
    /// and `20` is pass as a value, because there is only 1 value and 2 arguments,
    /// `min` must have its default value and `max` must receive the `20`.
    fn require_default_values<'a, I>(&self, args: &ArgumentList, iterator: &I) -> bool
        where I: Iterator<Item=&'a Token> + Clone {
        let contains_default_args = args.iter().any(|a| a.has_default_values());
        if contains_default_args {
            let available_values = iterator.clone().into_iter().take_while(|t| t.is_arg()).count();
            let mut required_values: usize = 0;

            for arg in args {
                // To avoid overflow
                match required_values.checked_add(arg.get_arg_count().max()) {
                    None => return false,
                    Some(n) => required_values = n
                }
            }

            return available_values < required_values;
        }

        false
    }
}

pub(crate) fn get_option_prefixed<'a>(
    context: &'a Context,
    command: &'a Command,
    prefix: &'a str,
    option: &'a str,
) -> Option<CommandOption> {
    if context.is_help(option) {
        if let Some(opt) = command.get_options().get(option){
            panic!("duplicated option: `{}`", opt.get_name());
        }

        if let Some(help) = context.help() {
            if matches!(help.kind(), HelpKind::Option | HelpKind::Any) {
                // Returns the `help` option.
                return Some(crate::help::to_option(context.help().unwrap().as_ref()));
            }
        }
    }

    if context.is_name_prefix(prefix) {
        return command.get_options().get_by_name(option).cloned();
    }

    if context.is_alias_prefix(prefix) {
        return command.get_options().get_by_alias(option).cloned();
    }

    None
}