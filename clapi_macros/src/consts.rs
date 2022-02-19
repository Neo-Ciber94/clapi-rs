// Constants values for `Attributes`

pub const COMMAND: &str = "command";
pub const SUBCOMMAND: &str = "subcommand";
pub const OPTION: &str = "option";
pub const ARG: &str = "arg";
pub const USAGE: &str = "usage";
pub const WITH_USAGE: &str = "with_usage";
pub const HELP: &str = "help";
pub const WITH_HELP: &str = "with_help";
pub const NAME: &str = "name";
pub const ALIAS: &str = "alias";
pub const VERSION: &str = "version";
pub const DESCRIPTION: &str = "description";
pub const PARENT: &str = "parent";
pub const MIN: &str = "min";
pub const MAX: &str = "max";
pub const DEFAULT: &str = "default";
pub const VALUES: &str = "values";
pub const HIDDEN: &str = "hidden";
pub const GLOBAL: &str = "global";
pub const MULTIPLE: &str = "multiple";
pub const REQUIRES_ASSIGN: &str = "requires_assign";
pub const FLAG: &str = "flag";
pub const ERROR: &str = "error";
pub const COMMAND_HELP: &str = "command_help";
pub const COMMAND_USAGE: &str = "command_usage";

pub fn is_clapi_attribute(path: &str) -> bool {
    is_command(path) || is_subcommand(path) || is_option(path) || is_arg(path)
}

pub fn is_command(path: &str) -> bool {
    matches!(
        path,
        "command" | "clapi::command" | "clapi::macros::command" | "clapi_macros::command"
    )
}

pub fn is_subcommand(path: &str) -> bool {
    matches!(
        path,
        "subcommand"
            | "clapi::subcommand"
            | "clapi::macros::subcommand"
            | "clapi_macros::subcommand"
    )
}

pub fn is_option(path: &str) -> bool {
    matches!(
        path,
        "option" | "clapi::option" | "clapi::macros::option" | "clapi_macros::option"
    )
}

pub fn is_arg(path: &str) -> bool {
    matches!(
        path,
        "arg" | "clapi::arg" | "clapi::macros::arg" | "clapi_macros::arg"
    )
}
