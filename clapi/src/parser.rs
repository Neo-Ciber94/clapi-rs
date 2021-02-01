#![allow(clippy::collapsible_if, clippy::len_zero)]
use std::borrow::Borrow;
use crate::args::ArgumentList;
use crate::command::Command;
use crate::context::Context;
use crate::error::{Error, ErrorKind, Result};
use crate::help::{HelpKind, Help};
use crate::option::{CommandOption, OptionList};
use crate::parse_result::ParseResult;
use crate::tokenizer::{Token, Tokenizer};
use crate::Argument;
use std::cell::Cell;

/// A command-line argument parser.
#[derive(Debug, Clone)]
pub struct Parser<'a> {
    context: &'a Context,
    cursor: Option<Cursor>,
    command: Option<Command>,
    options: Option<OptionList>,
    args: Option<ArgumentList>,
}

impl<'a> Parser<'a> {
    pub fn new(context: &'a Context) -> Self {
        Parser {
            context,
            cursor: None,
            command: None,
            options: Some(OptionList::new()),
            args: Some(ArgumentList::new()),
        }
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
        // If cursor is already set, reset the `Parser` state
        if self.cursor.is_some() {
            self.command = None;
            self.options = Some(OptionList::new());
            self.args = Some(ArgumentList::new());
        }

        // Parse the tokens using the current `Context`
        let tokens = Tokenizer.tokenize(self.context, args)?;

        // Constructs a `Cursor` using the tokens
        self.cursor = Some(Cursor::new(tokens));

        // Parse all the tokens
        self.parse_tokens()
    }

    fn parse_tokens(&mut self) -> Result<ParseResult> {
        // Parse executing command
        self.parse_executing_command()?;

        // Parse the commands options and its arguments
        self.parse_options()?;

        // Quick path: If the current parsing result contains a `help` command or option exit
        // due it consumes all the tokens in the `Cursor`
        if self.contains_help() {
            let command = self.command.take().unwrap();
            let options = self.options.take().unwrap();
            let args = self.args.take().unwrap();
            return Ok(ParseResult::new(command, options, args));
        }

        // Skip next `end of arguments` token (if any)
        if let Some(index) = self.cursor.as_ref().unwrap().remaining().iter().position(|t| t.is_eoo()) {
            // If there is arguments before `--` (end of arguments)
            // values are being passed to the last option which not exist.
            //
            // For example: 1 2 3 -- Hello World
            // This is a error because there is no option to pass 1 2 3
            if index > 0 {
                // Check Guide 10
                // https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html

                // We get the last argument to provide a hint of the error
                let value = self.cursor.as_ref()
                    .unwrap()
                    .next()
                    .cloned()
                    .unwrap()
                    .into_string();

                return Err(Error::new(
                    ErrorKind::InvalidArgument(value),
                    "there is no options that expect arguments",
                ));
            } else {
                self.cursor.as_ref().unwrap().next();
            }
        }

        // Check and set required options (if any)
        self.check_required_options()?;

        // Check and set options with default values (if any)
        self.set_default_options();

        // Parse the command arguments
        self.parse_args()?;

        // If there is arguments left and the current command takes no arguments is an error
        if self.cursor.as_ref().unwrap().peek().is_some() {
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

    fn parse_executing_command(&mut self) -> Result<()> {
        let cursor = self.cursor.as_ref().unwrap();
        let mut command = self.context.root();

        // If the next is `help [subcommand`
        if let Some(Token::Cmd(name)) = cursor.peek() {
            if is_help_command(&self.context, name) {
                return self.parse_help_command();
            }
        }

        while let Some(Token::Cmd(name)) = cursor.peek() {
            command = match command.find_subcommand(name.as_str()) {
                Some(x) => x,
                None => {
                    self.command = Some(command.clone());
                    return Err(Error::from(ErrorKind::UnexpectedCommand(name.clone())))
                }
            };

            cursor.next();
        }

        self.command = Some(command.clone());
        Ok(())
    }

    fn parse_options(&mut self) -> Result<()> {
        let cursor = self.cursor.as_ref().unwrap();
        let command = self.command.as_ref().unwrap();

        while let Some(Token::Opt(s)) = cursor.peek() {
            if is_help_option(&self.context, s) {
                return self.parse_help_option();
            }

            if let Some(option) = find_prefixed_option(&self.context, command, s) {
                // Consumes option token
                cursor.next();

                if option.is_assign_required() && option.take_args() {
                    if let Some(Token::Arg(arg)) = cursor.peek() {
                        let assign_op : char = *self.context.assign_operators().next().unwrap();
                        return Err(
                            Error::new(
                                ErrorKind::InvalidArgument(arg.clone()),
                                format!(
                                    "`{}` requires an assignment operator like `{}` for the arguments", s, assign_op
                                )
                            )
                        );
                    }
                }

                // Skips the assign operator if any
                if let Some(Token::AssignOp(_)) = cursor.peek() {
                    cursor.next();
                }

                if option.take_args() {
                    let mut option_args = ArgumentList::new();
                    let mut option_args_iter = option.get_args().iter().cloned().peekable();
                    let require_default_values = self.require_default_values(option.get_args());
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
                            if let Some(Token::Arg(value)) = cursor.peek() {
                                cursor.next();
                                values.push(value.clone());
                                count += 1;
                            } else {
                                break;
                            }
                        }

                        // If there is no more option args, check if there is an `end of arguments`
                        if option_args_iter.peek().is_none() {
                            if cursor.peek().map_or(false, |t| !t.is_option()) {
                                // Check Guide 10
                                // https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html
                                // If there is an `--` (end of arguments) we pass all the values
                                // before it to the last option as arguments (if any)
                                //
                                // Example: --numbers 1 2 3 -- hello world
                                // 1 2 3 are passed to the option `--numbers`
                                if let Some(mut index) = cursor.remaining().iter().position(|t| t.is_eoo()) {
                                    while index > 0 {
                                        let s = cursor.next().unwrap().clone().into_string();
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
                return Err(Error::from(ErrorKind::UnexpectedOption(s.clone())));
            }
        }

        Ok(())
    }

    fn parse_args(&mut self) -> Result<()> {
        let cursor = self.cursor.as_ref().unwrap();
        let command = self.command.as_ref().unwrap();
        let mut args_iter = command.get_args().iter().cloned().peekable();
        let require_default_values = self.require_default_values(command.get_args());
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
                    if let Some(Token::Arg(value)) = cursor.peek() {
                        cursor.next();
                        values.push(value.clone());
                        count += 1;
                    } else {
                        break;
                    }
                }
            } else {
                // If there is no `Argument`s left, pass the rest of the tokens as values
                while let Some(t) = cursor.next().cloned() {
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

    fn parse_help_command(&mut self) -> Result<()>{
        let cursor = self.cursor.as_ref().unwrap();

        if let Some(Token::Cmd(name)) = cursor.next() {
            debug_assert!(is_help_command(&self.context, name));

            let command = self.context.root().find_subcommand(name).unwrap();
            let mut args = ArgumentList::new();
            let mut arg = command.get_arg().unwrap().clone();
            let values = cursor.remaining()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            arg.set_values(values)?;
            args.add(arg).unwrap();

            // We already take all the remaining tokens
            cursor.move_to_end();

            // Sets the executing `help` command and the arguments
            self.command = Some(command.clone());
            self.args = Some(args);
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn parse_help_option(&mut self) -> Result<()> {
        let cursor = self.cursor.as_ref().unwrap();

        if let Some(Token::Opt(s)) = cursor.next() {
            debug_assert!(is_help_option(&self.context, s));

            let command = self.command.as_ref().unwrap();
            let option = find_prefixed_option(&self.context, command, s).unwrap();
            let mut args = ArgumentList::new();
            let mut arg = option.get_arg().unwrap().clone();

            if cursor.position() == 1 {
                // We take all the available values as arguments for the help
                let values = cursor.remaining()
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();

                arg.set_values(values)?;
            } else {
                // If the help is like: `[subcommand] --help` all the values before the `--help`
                // will be used as arguments
                let index = cursor.position();
                let values = cursor.tokens()[..index - 1]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();

                arg.set_values(values)?;
            }

            // Adds the single argument
            args.add(arg).unwrap();

            // Ignore the rest of tokens
            cursor.move_to_end();

            // Set all the values to the help `CommandOption`
            self.options.as_mut().unwrap().add(option.args(args)).unwrap();
            Ok(())
        } else {
            unreachable!()
        }
    }

    fn check_required_options(&self) -> Result<()> {
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
    fn require_default_values(&self, args: &ArgumentList) -> bool {
        let cursor = self.cursor.as_ref().unwrap();
        let contains_default_args = args.iter().any(|a| a.has_default_values());
        if contains_default_args {
            let available_values = cursor.remaining().iter().take_while(|t| t.is_arg()).count();
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

    fn contains_help(&self) -> bool {
        #[inline]
        fn contains_help_command(parser: &Parser, help: &dyn Help) -> bool {
            parser.command.as_ref().unwrap().get_name() == help.name()
        }

        #[inline]
        fn contains_help_option(parser: &Parser, help: &dyn Help) -> bool {
            let options = parser.options.as_ref().unwrap();
            options.contains(help.name())
                || help.alias().map_or(false, |s| options.contains(s))
        }

        if let Some(help) = self.context.help() {
            match help.kind(){
                HelpKind::Command => contains_help_command(self, help.as_ref()),
                HelpKind::Option => contains_help_option(self, help.as_ref()),
                HelpKind::Any => contains_help_command(self, help.as_ref())
                    || contains_help_option(self, help.as_ref())
            }

        } else {
            false
        }
    }
}

// A cursor over the tokens to parse
#[derive(Debug, Clone)]
struct Cursor {
    tokens: Vec<Token>,
    index: Cell<usize>,
}

impl Cursor {
    pub fn new(tokens: Vec<Token>) -> Self {
        Cursor {
            tokens,
            index: Cell::new(0),
        }
    }

    #[inline]
    pub fn tokens(&self) -> &[Token] {
        self.tokens.as_slice()
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.index.get()
    }

    #[inline]
    pub fn remaining(&self) -> &[Token] {
        &self.tokens[self.index.get()..]
    }

    #[inline]
    pub fn move_to_end(&self) {
        self.index.set(self.tokens.len())
    }

    #[inline]
    pub fn next(&self) -> Option<&Token> {
        let token = self.current();
        if token.is_some() {
            self.index.set(self.index.get() + 1);
        }
        token
    }

    #[inline]
    pub fn peek(&self) -> Option<&Token> {
        self.current()
    }

    fn current(&self) -> Option<&Token> {
        let tokens = self.tokens.as_slice();
        let index = self.index.get();

        if index >= tokens.len() {
            None
        } else {
            Some(&tokens[index])
        }
    }
}

fn find_prefixed_option<'a>(
    context: &'a Context,
    command: &'a Command,
    prefixed_option: &'a str,
) -> Option<CommandOption> {
    let unprefixed_option = context.trim_prefix(prefixed_option);

    // Check if the option is a `help`, like: `--help`
    if context.is_help(unprefixed_option) {
        // Check if the command already contains a `--help` defined
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

    // Check if the option is a `version`, like: `--version`
    if context.is_version(unprefixed_option) {
        return Some(crate::version::to_option(context.version().as_ref()));
    }

    // Finds and return the option from the context
    context.get_option(unprefixed_option).cloned()
}

fn is_help_command(context: &Context, command: &str) -> bool {
    if let Some(help) = context.help() {
        matches!(help.kind(), HelpKind::Any | HelpKind::Command) && help.name() == command
    } else {
        false
    }
}

fn is_help_option(context: &Context, option: &str) -> bool {
    if let Some(help) = context.help() {
        let name = context.trim_prefix(option);
        matches!(help.kind(), HelpKind::Any | HelpKind::Option)
            && (help.name() == name || help.alias().map_or(false, |s| s == name))
    } else {
        false
    }
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