#![allow(clippy::collapsible_if, clippy::len_zero)]
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
use crate::Argument;

/// A command-line argument parser.
#[derive(Debug, Clone)]
pub struct Parser<'a> {
    context: &'a Context,
    state: ParseState,
    command: Option<Command>,
    options: Option<OptionList>,
    args: Option<ArgumentList>,
}

/// State of the parser.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ParseState {
    /// The parser is not executed yet.
    Uninitialized,
    /// The parse operation finished successfully.
    Completed,
    /// The parse operation failed.
    Failed
}

impl<'a> Parser<'a> {
    pub fn new(context: &'a Context) -> Self {
        Parser {
            context,
            command: None,
            options: Some(OptionList::new()),
            args: Some(ArgumentList::new()),
            state: ParseState::Uninitialized,
        }
    }

    pub fn state(&self) -> ParseState {
        self.state
    }

    pub fn is_completed(&self) -> bool {
        self.state == ParseState::Completed
    }

    pub fn is_failed(&self) -> bool {
        self.state == ParseState::Failed
    }

    pub fn command(&self) -> Option<&Command> {
        self.command.as_ref()
    }

    pub fn options(&self) -> Option<&OptionList> {
        self.options.as_ref()
    }

    pub fn args(&self) -> Option<&ArgumentList> {
        self.args.as_ref()
    }

    pub fn parse<S, I>(&mut self, args: I) -> Result<ParseResult>
        where S: Borrow<str>,
              I: IntoIterator<Item = S> {
        // We prevent to run the parser twice due it maintain
        // the state of the last parse in case an error have occurred.
        if self.state != ParseState::Uninitialized {
            panic!("Parser have been used");
        }

        let tokens = Tokenizer
            .tokenize(&self.context, args)
            .map_err(|error| {
                self.state = ParseState::Failed;
                error
        })?;

        match self.parse_tokens(tokens) {
            Ok(result) => {
                self.state = ParseState::Completed;
                Ok(result)
            },
            Err(error) => {
                self.state = ParseState::Failed;
                Err(error)
            }
        }
    }

    fn parse_tokens(&mut self, tokens: Vec<Token>) -> Result<ParseResult> {
        // We takes an iterator over the tokens to parse, and parse the command, options and args,
        // the order of these operations matters due the commands are expected as:
        // <subcommands> <options> <option_arguments> <end of options> <command_arguments>
        let mut iterator = tokens.iter().peekable();

        // Parse executing command
        self.parse_executing_command(&mut iterator)?;

        // Parse the commands options and its arguments
        self.parse_options(&mut iterator)?;

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
        self.set_required_options()?;

        // Check and set options with default values (if any)
        self.set_default_options();

        // Parse the command arguments
        self.parse_args(&mut iterator)?;

        // If there is arguments left and the current command takes no arguments is an error
        if iterator.peek().is_some() {
            let command = self.command.as_ref().unwrap();
            return Err(Error::new(
                ErrorKind::InvalidArgumentCount,
                format!("`{}` takes no arguments", command.get_name()),
            ));
        }

        // Sets the command, options and arguments
        let command = self.command.take().unwrap();
        let options = self.options.take().unwrap();
        let args = self.args.take().unwrap();
        Ok(ParseResult::new(command, options, args))
    }

    fn parse_executing_command<'b, I>(&mut self, iterator: &mut Peekable<I>) -> Result<()> where I: Iterator<Item=&'b Token> {
        let mut command = self.context.root();
        while let Some(Token::Cmd(name)) = iterator.peek() {
            command = match command.find_subcommand(name.as_str()) {
                Some(x) => x,
                None => {
                    self.command = Some(command.clone());
                    return Err(Error::from(ErrorKind::UnrecognizedCommand(name.clone())))
                }
            };

            iterator.next();
        }

        self.command = Some(command.clone());
        Ok(())
    }

    fn parse_options<'b, I>(&mut self, iterator: &mut Peekable<I>) -> Result<()> where I: Iterator<Item=&'b Token> + Clone {
        let command = self.command.as_ref().unwrap();

        while let Some(Token::Opt(s)) = iterator.peek() {
            if let Some(option) = find_prefixed_option(&self.context, command, s) {
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
                                add_argument(&mut option_args, arg);

                                // This is just a flag, `Argument`S with default values already have
                                // the default value set
                                default_value_is_set = true;
                                continue;
                            }
                        }

                        let mut values = Vec::new();
                        let max_count = arg.get_values_count().max_or_default();
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
                        arg.set_values(values)?;
                        add_argument(&mut option_args, arg);
                    }

                    // Sets the option arguments
                    add_option(self.options.as_mut().unwrap(), option.args(option_args))?;
                } else {
                    // Adds the option
                    // SAFETY: `add_option` only fail with duplicated options that allow multiples,
                    // and takes args
                    add_option(self.options.as_mut().unwrap(), option).unwrap();
                }
            } else {
                return Err(Error::from(ErrorKind::UnrecognizedOption(s.clone())));
            }
        }

        Ok(())
    }

    fn parse_args<'b, I>(&mut self, iterator: &mut Peekable<I>) -> Result<()> where I: Iterator<Item=&'b Token> + Clone {
        let command = self.command.as_ref().unwrap();
        let mut args_iter = command.get_args().iter().cloned().peekable();
        let require_default_values = self.require_default_values(command.get_args(), iterator);
        let mut default_value_is_set = false;

        while let Some(mut arg) = args_iter.next() {
            let mut values = Vec::new();

            // We take the first `Argument` that required a default values.
            // Only 1 because multiple arguments with default values is no allowed.
            if require_default_values && !default_value_is_set {
                if arg.has_default_values() {
                    add_argument(self.args.as_mut().unwrap(), arg);

                    // This is just a flag, `Argument`S with default values already have
                    // the default value set
                    default_value_is_set = true;
                    continue;
                }
            }

            if args_iter.peek().is_some() {
                let max_count = arg.get_values_count().max_or_default();
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
                arg.set_values(values)?;
            }

            add_argument(self.args.as_mut().unwrap(), arg);
        }

        Ok(())
    }

    fn set_required_options(&self) -> Result<()> {
        let options = self.options.as_ref().unwrap();
        let command = self.command.as_ref().unwrap();
        let required_options = command
            .get_options().iter()
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

    fn set_default_options(&mut self) {
        let command = self.command.as_ref().unwrap();
        let default_options = command
            .get_options().iter()
            .filter(|o| o.get_args().iter().any(|a| a.has_default_values()));

        // Sets the options that takes default arguments
        for opt in default_options {
            if !self.options.as_ref().unwrap().contains(opt.get_name()) {
                // SAFETY: `add_option` only fail with duplicated options that allow multiples
                add_option(self.options.as_mut().unwrap(), opt.clone()).unwrap();
            }
        }
    }

    /// Returns `true` if one of the arguments need to have a default value.
    ///
    /// This is true when there is no enough values for the arguments.
    /// For example: the arguments `min` (default 0) and `max` are declared
    /// and `20` is pass as a value, because there is only 1 value and 2 arguments,
    /// `min` must have its default value and `max` must receive the `20`.
    fn require_default_values<'b, I>(&self, args: &ArgumentList, iterator: &I) -> bool
        where I: Iterator<Item=&'b Token> + Clone {
        let contains_default_args = args.iter().any(|a| a.has_default_values());
        if contains_default_args {
            let available_values = iterator.clone().into_iter().take_while(|t| t.is_arg()).count();
            let mut required_values: usize = 0;

            for arg in args {
                // To avoid overflow
                match required_values.checked_add(arg.get_values_count().max_or_default()) {
                    None => return false,
                    Some(n) => required_values = n
                }
            }

            return available_values < required_values;
        }

        false
    }
}

pub(crate) fn find_prefixed_option<'a>(
    context: &'a Context,
    command: &'a Command,
    prefixed_option: &'a str,
) -> Option<CommandOption> {
    let unprefixed_option = context.trim_prefix(prefixed_option);

    // Check if the option is a help, like: `--help`
    if context.is_help(unprefixed_option) {
        if let Some(opt) = command.get_options().get(unprefixed_option){
            panic!("duplicated option: `{}`", opt.get_name());
        }

        if let Some(help) = context.help() {
            if matches!(help.kind(), HelpKind::Option | HelpKind::Any) {
                // Returns the `help` option.
                return Some(crate::help::to_option(context.help().unwrap().as_ref()));
            }
        }
    }

    // Finds and return the option from the context
    context.get_option(unprefixed_option).cloned()
}

fn add_option(options: &mut OptionList, new_option: CommandOption) -> Result<()> {
    if new_option.allow_multiple() && options.contains(new_option.get_name()) {
        // If don't takes args is no-op
        if !new_option.take_args() {
            return Ok(());
        }

        let mut args = ArgumentList::new();
        let option = options.get(new_option.get_name()).unwrap();

        for arg in option.get_args() {
            let mut values = Vec::new();
            values.extend_from_slice(arg.get_values());
            let new_option_args = new_option.get_args()
                .get(arg.get_name())
                .unwrap();

            values.extend_from_slice(new_option_args.get_values());

            let mut new_arg = arg.clone();
            new_arg.set_values(values)?;

            // SAFETY: If `options` already contains the `option` which have no duplicates
            args.add(new_arg).unwrap();
        }

        options.add_or_replace(new_option.args(args));
        Ok(())
    } else {
        options.add(new_option).unwrap_or_else(|e| {
            panic!("option `{}` was specified multiple times but 1 was expected", e.get_name())
        });
        Ok(())
    }
}

fn add_argument(arguments: &mut ArgumentList, new_arg: Argument){
    arguments.add(new_arg).unwrap_or_else(|e| {
        panic!("duplicated argument: `{}`", e.get_name())
    });
}