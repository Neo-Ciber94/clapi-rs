use crate::context::Context;
use crate::error::{Error, ErrorKind, Result};
use crate::utils::Then;
use std::borrow::Borrow;

/// Represents a command-line token.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    // A command
    Cmd(String),
    // An option
    Opt(String, String),
    // An argument
    Arg(String),
    // End of options
    EOO,
}

const END_OF_OPTIONS : &'static str = "--";

impl Token {
    /// Returns `true` if the token is a command.
    pub fn is_command(&self) -> bool {
        match self {
            Token::Cmd(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if the token is an option.
    pub fn is_option(&self) -> bool {
        match self {
            Token::Opt(_, _) => true,
            _ => false,
        }
    }

    /// Returns `true` if the token is an argument.
    pub fn is_arg(&self) -> bool {
        match self {
            Token::Arg(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if the token represents an `end of options`.
    pub fn is_eoo(&self) -> bool {
        match self {
            Token::EOO => true,
            _ => false,
        }
    }

    /// Converts this `Token` into its `String` value.
    pub fn into_string(self) -> String {
        match self {
            Token::Cmd(s) => s,
            Token::Opt(_, s) => s,
            Token::Arg(s) => s,
            Token::EOO => String::from(END_OF_OPTIONS),
        }
    }
}

/// A trait to converts to tokens the given arguments.
pub trait Tokenizer<Args> {
    fn tokenize(&mut self, context: &Context, args: Args) -> Result<Vec<Token>>;
}

/// A default implementation of the `Tokenizer` trait.
#[derive(Default, Debug)]
pub struct DefaultTokenizer;
impl<'a, S, I> Tokenizer<I> for DefaultTokenizer
where
    S: Borrow<str> + 'a,
    I: IntoIterator<Item = S>,
{
    fn tokenize(&mut self, context: &Context, args: I) -> Result<Vec<Token>> {
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
                    && !context.is_option_prefixed(arg.borrow())
                {
                    tokens.push(Token::Cmd(arg.borrow().to_string()));
                    iterator.next();
                }

                break;
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

            if context.is_option_prefixed(value) {
                let OptionAndArgs {
                    prefix,
                    option,
                    args,
                } = try_split_option_and_args(context, value)?;

                tokens.push(Token::Opt(prefix, option.clone()));

                // Moves to the next value
                iterator.next();

                if let Some(args) = args {
                    tokens.extend(args.into_iter().map(|t| Token::Arg(t)));
                } else {
                    if let Some(opt) = current_command.get_options().get(option.as_str()) {
                        for arg in opt.get_args() {
                            let max_arg_count = arg.get_arg_count().max();
                            let mut count = 0;
                            while count < max_arg_count {
                                if let Some(value) = iterator.peek(){
                                    let s : &str = value.borrow();
                                    // If the token is prefixed as an option: exit
                                    if context.is_option_prefixed(s) || s == END_OF_OPTIONS{
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
                let s : String = value.borrow().to_string();
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
    prefix: String,
    option: String,
    args: Option<Vec<String>>,
}

fn try_split_option_and_args(context: &Context, value: &str) -> Result<OptionAndArgs> {
    // Check if the value contains a delimiter
    if let Some(delimiter) = context
        .arg_delimiters()
        .cloned()
        .find(|d| value.contains(*d))
    {
        let option_and_args = value
            .splitn(2, delimiter)
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        return if option_and_args.len() != 2 {
            Err(Error::from(ErrorKind::InvalidExpression))
        } else {
            let (prefix, option) =
                context
                    .trim_and_get_prefix(&option_and_args[0])
                    .then_apply(|(p, o)| {
                        (
                            p.unwrap().trim().to_string(),
                            o.trim_matches('"').trim().to_string(),
                        )
                    });

            let args = option_and_args[1]
                .split(",")
                .map(|s| s.trim_matches('"'))
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            if option.is_empty() {
                return Err(Error::new(ErrorKind::InvalidExpression, "option is empty"));
            }

            Ok(OptionAndArgs {
                prefix,
                option,
                args: Some(args),
            })
        };
    } else {
        let (prefix, option) = context.trim_and_get_prefix(value).then_apply(|(p, o)| {
            (
                p.unwrap().trim().to_string(),
                o.trim_matches('"').trim().to_string(),
            )
        });

        if option.is_empty() {
            return Err(Error::new(ErrorKind::InvalidExpression, "option is empty"));
        }

        Ok(OptionAndArgs {
            prefix,
            option,
            args: None,
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::arg_count::ArgCount;
//     use crate::args::Arguments;
//     use crate::command::Command;
//     use crate::command_line::into_arg_iterator;
//     use crate::option::CommandOption;
//
//     fn tokenize(value: &str) -> Result<Vec<Token>> {
//         let root =
//             Command::root()
//                 .option(CommandOption::new("version").alias("v"))
//                 .option(CommandOption::new("author").alias("a"))
//                 .subcommand(Command::new("echo").args(Arguments::new(1..)))
//                 .subcommand(
//                     Command::new("pick")
//                         .args(Arguments::new(ArgCount::new(1, 2)))
//                         .option(CommandOption::new("color").arg(
//                             Arguments::new(1).valid_values(&["red", "blue", "green"]),
//                         )),
//                 )
//                 .subcommand(
//                     Command::new("any").option(
//                         CommandOption::new("numbers")
//                             .required(true)
//                             .arg(Arguments::new(1..)),
//                     ),
//                 );
//
//         let values = into_arg_iterator(value);
//         let context = Context::new(root);
//         let mut tokenizer = DefaultTokenizer::default();
//         tokenizer.tokenize(&context, values)
//     }
//
//     #[test]
//     fn tokenize_test1() {
//         let tokens = tokenize("--version").unwrap();
//         assert_eq!(tokens[0], Token::Opt("--".to_owned(), "version".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test2() {
//         let tokens = tokenize("-a").unwrap();
//         assert_eq!(tokens[0], Token::Opt("-".to_owned(),"a".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test3() {
//         let tokens = tokenize("echo hello world").unwrap();
//         assert_eq!(tokens[0], Token::Cmd("echo".to_owned()));
//         assert_eq!(tokens[1], Token::Arg("hello".to_owned()));
//         assert_eq!(tokens[2], Token::Arg("world".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test4() {
//         let tokens = tokenize("pick one two").unwrap();
//         assert_eq!(tokens[0], Token::Cmd("pick".to_owned()));
//         assert_eq!(tokens[1], Token::Arg("one".to_owned()));
//         assert_eq!(tokens[2], Token::Arg("two".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test5() {
//         let tokens = tokenize("pick --color red one two").unwrap();
//         assert_eq!(tokens[0], Token::Cmd("pick".to_owned()));
//         assert_eq!(tokens[1], Token::Opt("--".to_owned(), "color".to_owned()));
//         assert_eq!(tokens[2], Token::Arg("red".to_owned()));
//         assert_eq!(tokens[3], Token::Arg("one".to_owned()));
//         assert_eq!(tokens[4], Token::Arg("two".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test6() {
//         let tokens = tokenize("pick --color=red one two").unwrap();
//         assert_eq!(tokens[0], Token::Cmd("pick".to_owned()));
//         assert_eq!(tokens[1], Token::Opt("--".to_owned(), "color".to_owned()));
//         assert_eq!(tokens[2], Token::Arg("red".to_owned()));
//         assert_eq!(tokens[3], Token::Arg("one".to_owned()));
//         assert_eq!(tokens[4], Token::Arg("two".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test7() {
//         let tokens = tokenize("pick --color:red one two").unwrap();
//         assert_eq!(tokens[0], Token::Cmd("pick".to_owned()));
//         assert_eq!(tokens[1], Token::Opt("--".to_owned(), "color".to_owned()));
//         assert_eq!(tokens[2], Token::Arg("red".to_owned()));
//         assert_eq!(tokens[3], Token::Arg("one".to_owned()));
//         assert_eq!(tokens[4], Token::Arg("two".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test8() {
//         let tokens = tokenize("pick --color:red one two").unwrap();
//         assert_eq!(tokens[0], Token::Cmd("pick".to_owned()));
//         assert_eq!(tokens[1], Token::Opt("--".to_owned(), "color".to_owned()));
//         assert_eq!(tokens[2], Token::Arg("red".to_owned()));
//         assert_eq!(tokens[3], Token::Arg("one".to_owned()));
//         assert_eq!(tokens[4], Token::Arg("two".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test9() {
//         let tokens = tokenize("any --numbers=3,1,2").unwrap();
//         assert_eq!(tokens[0], Token::Cmd("any".to_owned()));
//         assert_eq!(tokens[1], Token::Opt("--".to_owned(), "numbers".to_owned()));
//         assert_eq!(tokens[2], Token::Arg("3".to_owned()));
//         assert_eq!(tokens[3], Token::Arg("1".to_owned()));
//         assert_eq!(tokens[4], Token::Arg("2".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test10() {
//         let tokens = tokenize("any --numbers:3,1,2").unwrap();
//         assert_eq!(tokens[0], Token::Cmd("any".to_owned()));
//         assert_eq!(tokens[1], Token::Opt("--".to_owned(), "numbers".to_owned()));
//         assert_eq!(tokens[2], Token::Arg("3".to_owned()));
//         assert_eq!(tokens[3], Token::Arg("1".to_owned()));
//         assert_eq!(tokens[4], Token::Arg("2".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test11() {
//         let tokens = tokenize("--version --author").unwrap();
//         assert_eq!(tokens[0], Token::Opt("--".to_owned(), "version".to_owned()));
//         assert_eq!(tokens[1], Token::Opt("--".to_owned(), "author".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test12() {
//         let tokens = tokenize("any --numbers 1 2 3 -- red").unwrap();
//         assert_eq!(tokens[0], Token::Cmd("any".to_owned()));
//         assert_eq!(tokens[1], Token::Opt("--".to_owned(), "numbers".to_owned()));
//         assert_eq!(tokens[2], Token::Arg("1".to_owned()));
//         assert_eq!(tokens[3], Token::Arg("2".to_owned()));
//         assert_eq!(tokens[4], Token::Arg("3".to_owned()));
//         assert_eq!(tokens[5], Token::EOO);
//         assert_eq!(tokens[6], Token::Arg("red".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test13() {
//         let tokens = tokenize("any --numbers=1,2,3, 4").unwrap();
//         assert_eq!(tokens[0], Token::Cmd("any".to_owned()));
//         assert_eq!(tokens[1], Token::Opt("--".to_owned(), "numbers".to_owned()));
//         assert_eq!(tokens[2], Token::Arg("1".to_owned()));
//         assert_eq!(tokens[3], Token::Arg("2".to_owned()));
//         assert_eq!(tokens[4], Token::Arg("3".to_owned()));
//         assert_eq!(tokens[5], Token::Arg("4".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test14() {
//         let tokens = tokenize("any --numbers::1,:2,:3,:4").unwrap();
//         assert_eq!(tokens[0], Token::Cmd("any".to_owned()));
//         assert_eq!(tokens[1], Token::Opt("--".to_owned(), "numbers".to_owned()));
//         assert_eq!(tokens[2], Token::Arg(":1".to_owned()));
//         assert_eq!(tokens[3], Token::Arg(":2".to_owned()));
//         assert_eq!(tokens[4], Token::Arg(":3".to_owned()));
//         assert_eq!(tokens[5], Token::Arg(":4".to_owned()));
//     }
//
//     #[test]
//     fn tokenize_test15() {
//         let tokens = tokenize("any --numbers \". 1\" \". 2\" \". 3\"").unwrap();
//         assert_eq!(tokens[0], Token::Cmd("any".to_owned()));
//         assert_eq!(tokens[1], Token::Opt("--".to_owned(), "numbers".to_owned()));
//         assert_eq!(tokens[2], Token::Arg(". 1".to_owned()));
//         assert_eq!(tokens[3], Token::Arg(". 2".to_owned()));
//         assert_eq!(tokens[4], Token::Arg(". 3".to_owned()));
//     }
//
//     #[test]
//     fn tokenizer_ok_test() {
//         assert!(tokenize("").is_ok());
//         assert!(tokenize(" ").is_ok());
//         assert!(tokenize("--").is_ok());
//         assert!(tokenize("--all").is_ok());
//         assert!(tokenize("create").is_ok());
//         assert!(tokenize("create --path=hello").is_ok());
//     }
//
//     #[test]
//     fn tokenizer_error_test() {
//         assert!(tokenize("-").is_err());
//     }
// }
