// Constants values for `Attributes`

pub const COMMAND: &'static str = "command";
pub const SUBCOMMAND: &'static str = "subcommand";
pub const OPTION: &'static str = "option";
pub const ARG: &'static str = "arg";
pub const HELP: &'static str = "help";
pub const NAME: &'static str = "name";
pub const ALIAS: &'static str = "alias";
pub const VERSION: &'static str = "version";
pub const DESCRIPTION: &'static str = "description";
pub const PARENT: &'static str = "parent";
pub const ABOUT: &'static str = "about";
pub const MIN: &'static str = "min";
pub const MAX: &'static str = "max";
pub const DEFAULT: &'static str = "default";

pub fn is_clapi_attribute(path: &str) -> bool {
    is_command(path) || is_subcommand(path) || is_option(path) || is_arg(path) || is_help(path)
}

pub fn is_command(path: &str) -> bool {
    match path {
        "command" | "clapi::command" | "clapi::macros::command" | "clapi_macros::command" => true,
        _ => false,
    }
}

pub fn is_subcommand(path: &str) -> bool {
    match path {
        "subcommand"
        | "clapi::subcommand"
        | "clapi::macros::subcommand"
        | "clapi_macros::subcommand" => true,
        _ => false,
    }
}

pub fn is_option(path: &str) -> bool {
    match path {
        "option" | "clapi::option" | "clapi::macros::option" | "clapi_macros::option" => true,
        _ => false,
    }
}

pub fn is_arg(path: &str) -> bool {
    match path {
        "arg" | "clapi::arg" | "clapi::macros::arg" | "clapi_macros::arg" => true,
        _ => false,
    }
}

pub fn is_help(path: &str) -> bool {
    match path {
        "help" | "clapi::help" | "clapi::macros::help" | "clapi_macros::help" => true,
        _ => false,
    }
}
