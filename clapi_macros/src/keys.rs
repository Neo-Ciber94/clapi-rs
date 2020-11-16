// Constants values

pub const COMMAND: &'static str = "command";
pub const SUBCOMMAND: &'static str = "subcommand";
pub const OPTION: &'static str = "option";
pub const ARG: &'static str = "arg";
pub const NAME : &'static str = "name";
pub const ALIAS: &'static str = "alias";
pub const VERSION: &'static str = "version";
pub const DESCRIPTION: &'static str = "description";
pub const HELP: &'static str = "help";
pub const MIN: &'static str = "min";
pub const MAX: &'static str = "max";
pub const DEFAULT: &'static str = "default";

pub fn is_clapi_attribute(path: &str) -> bool {
    is_command(path)
    || is_subcommand(path)
    || is_option(path)
    || is_arg(path)
}

pub fn is_command(path: &str) -> bool {
    match path {
        "command" | "clapi::command" | "clapi_macro::command" => true,
        _ => false
    }
}

pub fn is_subcommand(path: &str) -> bool {
    match path {
        "subcommand" | "clapi::subcommand" | "clapi_macro::subcommand" => true,
        _ => false
    }
}

pub fn is_option(path: &str) -> bool {
    match path {
        "option" | "clapi::option" | "clapi_macro::option" => true,
        _ => false
    }
}

pub fn is_arg(path: &str) -> bool {
    match path {
        "arg" | "clapi::arg" | "clapi_macro::arg" => true,
        _ => false
    }
}