use crate::CommandOption;

/// Trait template for the option `version`.
pub trait VersionProvider {
    /// Returns the name of the version option.
    fn name(&self) -> &str;

    /// Returns the alias of the version option.
    fn alias(&self) -> Option<&str> {
        None
    }

    /// Returns the description of the version option.
    fn description(&self) -> Option<&str> {
        None
    }
}

/// Default implementation of `VersionProvider`.
pub struct DefaultVersionProvider;
impl VersionProvider for DefaultVersionProvider {
    fn name(&self) -> &str {
        "version"
    }

    fn alias(&self) -> Option<&str> {
        Some("v")
    }

    fn description(&self) -> Option<&str> {
        Some("Shows the version of the app")
    }
}

/// Constructs a `CommandOption` using the given `VersionProvider` data.
pub(crate) fn to_option(version: &dyn VersionProvider) -> CommandOption {
    let mut option = CommandOption::new(version.name());

    if let Some(alias) = version.alias() {
        option = option.alias(alias);
    }

    if let Some(description) = version.description() {
        option = option.description(description);
    }

    option
}