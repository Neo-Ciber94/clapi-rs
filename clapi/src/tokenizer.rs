use crate::context::Context;
use crate::error::{Error, ErrorKind, Result};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};

/// Represents a command-line token.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    // A command
    Cmd(String),
    // A prefixed option
    Opt(String),
    // An argument
    Arg(String),
    // End of options
    EOO,
    // Option assign operator
    AssignOp(char)
}

const END_OF_OPTIONS: &str = "--";

impl Token {
    /// Returns `true` if the token is a command.
    pub fn is_command(&self) -> bool {
        matches!(self, Token::Cmd(_))
    }

    /// Returns `true` if the token is an option.
    pub fn is_option(&self) -> bool {
        matches!(self, Token::Opt(_))
    }

    /// Returns `true` if the token is an argument.
    pub fn is_arg(&self) -> bool {
        matches!(self, Token::Arg(_))
    }

    /// Returns `true` if the token represents an `end of options`.
    pub fn is_eoo(&self) -> bool {
        matches!(self, Token::EOO)
    }

    pub fn is_assign_op(&self) -> bool {
        matches!(self, Token::AssignOp(_))
    }

    /// Returns a `String` representation of this `Token`.
    pub fn into_string(self) -> String {
        match self {
            Token::Cmd(s) => s,
            Token::Opt(s) => s,
            Token::Arg(s) => s,
            Token::EOO => String::from(END_OF_OPTIONS),
            Token::AssignOp(c) => c.to_string(),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Cmd(name) => write!(f, "{}", name),
            Token::Opt(name) => write!(f, "{}", name),
            Token::Arg(name) => write!(f, "{}", name),
            Token::EOO => write!(f, "{}", END_OF_OPTIONS),
            Token::AssignOp(c) => write!(f, "{}", c),
        }
    }
}

/// A converts a collection of `String`s to `Token`s.
#[derive(Debug)]
pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize<S, I>(&self, context: &Context, args: I) -> Result<Vec<Token>>
        where S: Borrow<str>,
              I: IntoIterator<Item = S> {
        let mut iterator = args
            .into_iter()
            .filter(|s| !s.borrow().is_empty())
            .peekable();

        // Quick path
        if iterator.peek().is_none() {
            return Ok(Vec::new());
        }

        let mut tokens = Vec::new();
        let mut current_command = context.root();
        let mut has_end_of_options = false;

        // Finds the executing command
        if iterator.peek().map_or(false, |s| crate::is_help_command(context, s.borrow())) {
            let s = iterator.next().unwrap().borrow().to_string();
            tokens.push(Token::Cmd(s))
        } else {
            while let Some(arg) = iterator.peek() {
                if let Some(child) = current_command.find_subcommand(arg.borrow()) {
                    current_command = child;
                    tokens.push(Token::Cmd(child.get_name().to_string()));
                    iterator.next();
                } else {
                    // If the current don't take args, have subcommands and is not an option
                    // the next should be an unknown subcommand
                    if !current_command.take_args()
                        && current_command.get_children().len() > 0
                        && !is_prefixed_option(context, arg.borrow())
                    {
                        tokens.push(Token::Cmd(arg.borrow().to_string()));
                        iterator.next();
                    }

                    break;
                }
            }
        }

        // Check for options
        while let Some(arg) = iterator.peek() {
            let value: &str = arg.borrow();

            // End of the options
            if value == END_OF_OPTIONS {
                tokens.push(Token::EOO);
                has_end_of_options = true;
                iterator.next();
                break;
            }

            if is_prefixed_option(context, value) {
                let OptionAndArgs {
                    prefixed_option,
                    args,
                    assign_op,
                } = try_split_option_and_args(context, value)?;

                // Moves to the next value
                iterator.next();

                // Adds the option
                tokens.push(Token::Opt(prefixed_option.clone()));

                // Adds the assign operator if any
                if let Some(c) = assign_op {
                    tokens.push(Token::AssignOp(c));
                }

                if let Some(args) = args {
                    tokens.extend(args.into_iter().map(Token::Arg));
                } else if let Some(opt) = current_command
                    .get_options()
                    .get(context.trim_prefix(&prefixed_option)) {

                    for arg in opt.get_args() {
                        let max_arg_count = arg.get_values_count().max_or_default();
                        let mut count = 0;
                        while count < max_arg_count {
                            if let Some(value) = iterator.peek() {
                                let s: &str = value.borrow();
                                // If the token is prefixed as an option: exit
                                if is_prefixed_option(context, s) || s == END_OF_OPTIONS {
                                    break;
                                } else {
                                    // Adds the next argument
                                    tokens.push(Token::Arg(s.to_string()));
                                    iterator.next();
                                    count += 1;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }

        if has_end_of_options {
            // The rest if considered arguments
            tokens.extend(iterator.map(|s| Token::Arg(s.borrow().to_string())));
        } else {
            for value in iterator {
                let s: String = value.borrow().to_string();
                if s == END_OF_OPTIONS && !has_end_of_options {
                    tokens.push(Token::EOO);
                    has_end_of_options = true;
                } else {
                    tokens.push(Token::Arg(s));
                }
            }
        }

        Ok(tokens)
    }
}

struct OptionAndArgs {
    prefixed_option: String,
    args: Option<Vec<String>>,
    assign_op: Option<char>,
}

fn try_split_option_and_args(context: &Context, value: &str) -> Result<OptionAndArgs> {
    // Check if the value contains an assign operator like: --times=1
    if let Some(assign_op) = context.assign_operators().cloned().find(|d| value.contains(*d)) {
        let option_and_args = value
            .split(assign_op)
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();

        // We expect 2 parts: `option` = `arg1`
        return if option_and_args.len() != 2 {
            Err(Error::from(ErrorKind::InvalidExpression))
        } else {
            // We use the unprefixed option to do checks
            let unprefixed_option = context.trim_prefix(&option_and_args[0]);

            let args = option_and_args[1]
                .split(context.delimiter())
                .map(|s| s.trim_matches('"'))
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            // Error when: `=1,2,3`
            if unprefixed_option.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidExpression,
                    "no option specified",
                ));
            }

            // Error when: `--option=`
            if args.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidExpression,
                    "no arguments specified",
                ));
            }

            // Error when: `--option=1,,,3`
            if args.iter().any(|s| s.is_empty()) {
                return Err(Error::new(
                    ErrorKind::InvalidExpression,
                    value,
                ));
            }

            Ok(OptionAndArgs {
                prefixed_option: option_and_args[0].clone(),
                args: Some(args),
                assign_op: Some(assign_op)
            })
        };
    } else {
        if context.trim_prefix(value).is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidExpression,
                "no option specified",
            ));
        }

        Ok(OptionAndArgs {
            prefixed_option: value.to_owned(),
            args: None,
            assign_op: None,
        })
    }
}

// Returns `true` if the specified value starts with an option prefix.
fn is_prefixed_option(context: &Context, value: &str) -> bool {
    context.name_prefixes()
        .chain(context.alias_prefixes())
        .any(|prefix| value.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{split_into_args, Argument, Command, CommandOption};

    fn tokenize(command: Command, value: &str) -> crate::Result<Vec<Token>> {
        let context = Context::new(command);
        Tokenizer.tokenize(&context, split_into_args(value))
    }

    #[test]
    fn tokenize_test() {
        let command = Command::new("MyApp")
            .arg(Argument::one_or_more("args"))
            .option(CommandOption::new("enable").alias("e"))
            .option(CommandOption::new("range").arg(Argument::with_name("range").values_count(1..=2)))
            .subcommand(Command::new("version"));

        assert_eq!(tokenize(command.clone(), "").unwrap(), Vec::new());

        let tokens1 = tokenize(command.clone(), "--range 1 -e").unwrap();
        assert_eq!(tokens1.len(), 3);
        assert_eq!(tokens1[0], Token::Opt("--range".to_owned()));
        assert_eq!(tokens1[1], Token::Arg("1".to_owned()));
        assert_eq!(tokens1[2], Token::Opt("-e".to_owned()));

        let tokens2 = tokenize(command.clone(), "version").unwrap();
        assert_eq!(tokens2.len(), 1);
        assert_eq!(tokens2[0], Token::Cmd("version".to_owned()));

        let tokens3 = tokenize(command.clone(), "--range 0 10 -- a b c").unwrap();
        assert_eq!(tokens3.len(), 7);
        assert_eq!(tokens3[0], Token::Opt("--range".to_owned()));
        assert_eq!(tokens3[1], Token::Arg("0".to_owned()));
        assert_eq!(tokens3[2], Token::Arg("10".to_owned()));
        assert_eq!(tokens3[3], Token::EOO);
        assert_eq!(tokens3[4], Token::Arg("a".to_owned()));
        assert_eq!(tokens3[5], Token::Arg("b".to_owned()));
        assert_eq!(tokens3[6], Token::Arg("c".to_owned()));
    }

    #[test]
    fn tokenize_test2() {
        let command = Command::new("MyApp")
            .arg(Argument::zero_or_one("values"))
            .option(
                CommandOption::new("times")
                    .alias("t")
                    .arg(Argument::with_name("times")),
            )
            .option(
                CommandOption::new("numbers")
                    .alias("n")
                    .arg(Argument::zero_or_one("N")),
            );

        let tokens1 = tokenize(command.clone(), "-t=1 --numbers=2,4,6 --").unwrap();
        assert_eq!(tokens1.len(), 9);
        assert_eq!(tokens1[0], Token::Opt("-t".to_owned()));
        assert_eq!(tokens1[1], Token::AssignOp('='));
        assert_eq!(tokens1[2], Token::Arg("1".to_owned()));
        assert_eq!(tokens1[3], Token::Opt("--numbers".to_owned()));
        assert_eq!(tokens1[4], Token::AssignOp('='));
        assert_eq!(tokens1[5], Token::Arg("2".to_owned()));
        assert_eq!(tokens1[6], Token::Arg("4".to_owned()));
        assert_eq!(tokens1[7], Token::Arg("6".to_owned()));
        assert_eq!(tokens1[8], Token::EOO);
    }

    #[test]
    fn invalid_expression_test() {
        let command = Command::new("MyApp")
            .arg(Argument::zero_or_one("values"))
            .option(
                CommandOption::new("times")
                    .alias("t")
                    .arg(Argument::with_name("times")),
            )
            .option(
                CommandOption::new("numbers")
                    .alias("n")
                    .arg(Argument::zero_or_one("N")),
            );

        // Err
        assert!(tokenize(command.clone(), "-").is_err());
        assert!(tokenize(command.clone(), "--numbers=").is_err());
        assert!(tokenize(command.clone(), "--numbers=,").is_err());
        assert!(tokenize(command.clone(), "--numbers=1,,,2").is_err());
        assert!(tokenize(command.clone(), "--numbers=1,2,3,").is_err());
        assert!(tokenize(command.clone(), "--numbers=,1,2,3").is_err());
    }
}
