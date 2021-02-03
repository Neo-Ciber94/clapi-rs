
/// Returns this `crate` name.
///
/// # Panics
/// Panics if package `name` is no defined.
#[macro_export]
macro_rules! crate_name {
    () => {
        option_env!("CARGO_PKG_NAME").expect("package `name` is not defined")
    };
}

/// Returns this `crate` description.
///
/// # Panics
/// Panics if package `description` is no defined.
#[macro_export]
macro_rules! crate_description {
    () => {
        option_env!("CARGO_PKG_DESCRIPTION").expect("package `description` is not defined")
    };
}

/// Returns this `crate` version.
///
/// # Panics
/// Panics if package `version` is no defined.
#[macro_export]
macro_rules! crate_version {
    () => {
        option_env!("CARGO_PKG_VERSION").expect("package `version` is not defined")
    };
}

/// Constructs a `CommandLine` app.
///
/// You use the `@subcommand`, `@option` and `@arg` tags to create subcommand, option and args
/// respectively. A list of the tags and its properties:
/// - `@subcommand` : description, usage, help, handler, hidden, @subcommand, @option and @arg.
/// - `@option` : description, alias, required, multiple, requires_assign and @arg.
/// - `@arg` : description, values, default, count, validator and type,
///
/// # Usage
/// To create the app start with:
/// * `clapi::app! { => ... }`
/// * `clapi::app! { AppName => ... }`
/// * `clapi::app! { "AppName" => ... }`
///
/// This is the root of the app where all the tags and properties are declared,
/// these are declared as `(property => value)`.
/// * For example:
/// ```
/// clapi::app! { MyApp =>
///     (description => "This is an app")
/// };
/// ```
///
/// And the tags like `@subcommand`, `@option` and `@arg`,
/// must contain a name either as an identifier or string literal for example:
/// * `(@subcommand version => ...)`
/// * `(@option "enable" => ...)`
///
/// Each tag contains its own properties, check the list above.
///
/// # Example
/// ```
/// clapi::app!{ MyApp =>
///     (description => "App to sum values")
///     (usage => "USAGE: \n command [--times] <values...>")
///     (@arg values =>
///         (count => 1..)
///         (type => i64)
///     )
///     (@option times =>
///         (description => "Number of times to sum the values")
///         (@arg =>
///             (type => u64)
///             (default => 1)
///         )
///     )
///     (handler (times: u64, ...values: Vec<i64>) => {
///         let times = times as i64;
///         let total : i64 = values.iter().sum();
///         println!("{}", total * times);
///     })
/// };
/// ```
#[macro_export]
macro_rules! app {
    //////////////////////////////////////////////////////////////////////
    // This is the entry point to create a `CommandLine`                //
    //////////////////////////////////////////////////////////////////////

    // Create a `CommandLine` with `Command::root()`
    (=> $($rest:tt)*) => {{
        $crate::CommandLine::new(
            $crate::app!{
                @command ($crate::Command::root()) $($rest)*
            }
        )
    }};

    // Create a `CommandLine` with `Command::new(command_name)`
    ($command_name:ident => $($rest:tt)*) => {{
        $crate::CommandLine::new(
            $crate::app!{
                @command ($crate::Command::new(stringify!($command_name))) $($rest)*
            }
        )
    }};

    // Create a `CommandLine` with `Command::new(command_name)`
    ($command_name:expr => $($rest:tt)*) => {{
        $crate::CommandLine::new(
            $crate::app!{
                @command ($crate::Command::new($command_name)) $($rest)*
            }
        )
    }};

    // Command fallthrough
    (@command ($builder:expr)) => { $builder };

    // Command `description`:
    // clapi::app! { MyApp => (description => ... ) }
    (@command ($builder:expr) (description => $description:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.description($description)) $($tt)*
        }
    };

    // Command `usage`:
    // clapi::app! { MyApp => (usage => ... ) }
    (@command ($builder:expr) (usage => $usage:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.usage($usage)) $($tt)*
        }
    };

    // Command `help`:
    // clapi::app! { MyApp => (help => ... ) }
    (@command ($builder:expr) (help => $help:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.help($help)) $($tt)*
        }
    };

    // Command `hidden`:
    // clapi::app! { MyApp => (hidden => ... ) }
    (@command ($builder:expr) (hidden => $hidden:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.hidden($hidden)) $($tt)*
        }
    };

    // Command handler with `OptionList` and `ArgumentList` with a block.
    // clapi::app! { MyApp => (handler (opts, args) => { ... } ) }
    (@command ($builder:expr) (handler ($options:ident, $arguments:ident) => $block:block) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|$options, $arguments|{
                $block
                Ok(())
            })) $($tt)*
        }
    };

    // Command handler with `OptionList` and `ArgumentList` with a single expression.
    // clapi::app! { MyApp => (handler (opts, args) => ... ) }
    (@command ($builder:expr) (handler ($options:ident, $arguments:ident) => $expr:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|$options, $arguments|{
                $expr;
                Ok(())
            })) $($tt)*
        }
    };

    // Command handler with typed arguments with a block.
    // clapi::app! { MyApp => (handler (...argument : type) => { ... } ) }
    (@command ($builder:expr) (handler (...$($arg_name:ident: $arg_type:ty),+) => $block:block) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|options, arguments|{
                $(
                    let $arg_name : $arg_type = $crate::declare_argument_var!(arguments, $arg_name: $arg_type);
                )+
                $block
                Ok(())
            })) $($tt)*
        }
    };

    // Command handler with typed arguments with a single expression.
    // clapi::app! { MyApp => (handler (...argument : type) => ... ) }
    (@command ($builder:expr) (handler (...$($arg_name:ident: $arg_type:ty),+) => $expr:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|options, arguments|{
                $(
                    let $arg_name : $arg_type = $crate::declare_argument_var!(arguments, $arg_name: $arg_type);
                )+
                $expr;
                Ok(())
            })) $($tt)*
        }
    };

    // Command handler with typed options and arguments with a block.
    // clapi::app! { MyApp => (handler (option : type, ...argument : type) => { ... } ) }
    (@command ($builder:expr) (handler ($($name:ident : $ty:ty),+ $(,...$($arg_name:ident: $arg_type:ty),+)?) => $block:block) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|options, arguments|{
                #[cfg(debug_assertions)]
                #[allow(unused_variables)]
                fn assert_non_duplicate_arguments($($name: $ty),+ $(,$($arg_name: $arg_type),+)?){}

                $(
                    let $name : $ty = $crate::declare_option_var!(options, $name: $ty);
                )+
                $(
                    $(
                        let $arg_name : $arg_type = $crate::declare_argument_var!(arguments, $arg_name: $arg_type);
                    )+
                )?
                $block
                Ok(())
            })) $($tt)*
        }
    };

    // Command handler with typed options and arguments with a single expression.
    // clapi::app! { MyApp => (handler (option : type, ...argument : type) => ... ) }
    (@command ($builder:expr) (handler ($($name:ident : $ty:ty),+ $(,...$($arg_name:ident: $arg_type:ty),+)?) => $expr:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|options, arguments|{
                #[cfg(debug_assertions)]
                #[allow(unused_variables)]
                fn assert_non_duplicate_arguments($($name: $ty),+ $(,$($arg_name: $arg_type),+)?){}

                $(
                    let $name : $ty = $crate::declare_option_var!(options, $name: $ty);
                )+
                $(
                    $(
                        let $arg_name : $arg_type = $crate::declare_argument_var!(arguments, $arg_name: $arg_type);
                    )+
                )?
                $expr;
                Ok(())
            })) $($tt)*
        }
    };

    // Command handler with no args with a block.
    // clapi::app! { MyApp => (handler () => { ... } ) }
    (@command ($builder:expr) (handler () => $block:block) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|_options, _arguments|{
                $block
                Ok(())
            })) $($tt)*
        }
    };

    // Command handler with no args with a single expression.
    // clapi::app! { MyApp => (handler () => ... ) }
    (@command ($builder:expr) (handler () => $expr:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|_options, _arguments|{
                $expr;
                Ok(())
            })) $($tt)*
        }
    };

    // Command handler with no args with a block.
    // clapi::app! { MyApp => (handler => { ... } ) }
    (@command ($builder:expr) (handler => $block:block) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|_options, _arguments|{
                $block
                Ok(())
            })) $($tt)*
        }
    };

    // Command handler with no args with a single expression.
    // clapi::app! { MyApp => (handler => ...) }
    (@command ($builder:expr) (handler => $expr:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|_options, _arguments|{
                $expr;
                Ok(())
            })) $($tt)*
        }
    };

    // Subcommand
    // clapi::app! { (@subcommand child => ( ... ) }
    (@command ($builder:expr) (@subcommand $command_name:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.subcommand(
                $crate::app!{ @command ($crate::Command::new(stringify!($command_name))) $($($rest)+)? }
            )) $($tt)*
        }
    };

    // Subcommand
    // clapi::app! { (@subcommand "child" => ( ... ) }
    (@command ($builder:expr) (@subcommand $command_name:expr $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.subcommand(
                $crate::app!{ @command ($crate::Command::new($command_name)) $($($rest)+)? }
            )) $($tt)*
        }
    };

    // Option
    // clapi::app! { (@option test => ( ... )) }
    (@command ($builder:expr) (@option $option_name:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.option(
                $crate::app!{ @option ($crate::CommandOption::new(stringify!($option_name))) $($($rest)+)? }
            )) $($tt)*
        }
    };

    // Option
    // clapi::app! { (@option "test" => ( ... )) }
    (@command ($builder:expr) (@option $option_name:expr $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.option(
                $crate::app!{ @option ($crate::CommandOption::new($option_name)) $($($rest)+)? }
            )) $($tt)*
        }
    };

    // Option fallthrough
    (@option ($option_builder:expr)) => { $option_builder };

    // Option argument
    // clapi::app! { (@option => (@arg value => ( ... )) }
    (@option ($option_builder:expr) (@arg $arg_name:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @option
            ($option_builder.arg(
                $crate::app!{ @arg ($crate::Argument::with_name(stringify!($arg_name))) $($($rest)+)? }
            )) $($tt)*
        }
    };

    // Option with named argument
    // clapi::app! { (@option => (@arg "value" => ( ... )) }
    (@option ($option_builder:expr) (@arg $arg_name:expr $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @option
            ($option_builder.arg(
                $crate::app!{ @arg ($crate::Argument::with_name($arg_name)) $($($rest)+)? }
            )) $($tt)*
        }
    };

    // Option argument
    // clapi::app! { (@option => (@arg => ( ... )) }
    (@option ($option_builder:expr) (@arg $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @option
            ($option_builder.arg(
                $crate::app!{ @arg ($crate::Argument::new()) $($($rest)+)? }
            )) $($tt)*
        }
    };

    // Option description
    // clapi::app! { (@option => (description => ... ) ) }
    (@option ($option_builder:expr) (description => $description:expr) $($tt:tt)*) => {
        $crate::app!{
            @option ($option_builder.description($description)) $($tt)*
        }
    };

    // Option required
    // clapi::app! { (@option => (required => ... ) ) }
    (@option ($option_builder:expr) (required => $required:expr) $($tt:tt)*) => {
        $crate::app!{
            @option ($option_builder.required($required)) $($tt)*
        }
    };

    // Option multiple
    // clapi::app! { (@option => (multiple => ... ) ) }
    (@option ($option_builder:expr) (multiple => $multiple:expr) $($tt:tt)*) => {
        $crate::app!{
            @option ($option_builder.multiple($multiple)) $($tt)*
        }
    };

    // Option hidden
    // clapi::app! { (@option => (hidden => ... ) ) }
    (@option ($option_builder:expr) (hidden => $hidden:expr) $($tt:tt)*) => {
        $crate::app!{
            @option ($option_builder.hidden($hidden)) $($tt)*
        }
    };

    // Option requires assign
    // clapi::app! { (@option => (requires_assign => ... ) ) }
    (@option ($option_builder:expr) (requires_assign => $requires_assign:expr) $($tt:tt)*) => {
        $crate::app!{
            @option ($option_builder.requires_assign($requires_assign)) $($tt)*
        }
    };

    // Option aliases
    // clapi::app! { (@option => (alias => ... ) ) }
    (@option ($option_builder:expr) (alias => $($alias:expr),+) $($tt:tt)*) => {
        $crate::app!{
            @option
            ($option_builder$(.alias($alias))+) $($tt)*
        }
    };

    // Command argument
    // clapi::app! { (@arg value => ( ... ) ) }
    (@command ($builder:expr) (@arg $arg_name:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.arg(
                $crate::app!{ @arg ($crate::Argument::with_name(stringify!($arg_name))) $($($rest)+)? }
            )) $($tt)*
        }
    };

    // Command argument
    // clapi::app! { (@arg "value" => ( ... ) ) }
    (@command ($builder:expr) (@arg $arg_name:expr $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.arg(
                $crate::app!{ @arg ($crate::Argument::with_name($arg_name)) $($($rest)+)? }
            )) $($tt)*
        }
    };

    // Argument fallthrough
    (@arg ($arg_builder:expr)) => { $arg_builder };

    // Argument value count
    // clapi::app! { (@arg => (count => 1..) }
    (@arg ($arg_builder:expr) (count => $count:expr) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.values_count($count)) $($tt)*
        }
    };

    // Argument description
    // clapi::app! { (@arg => (description => ... ) }
    (@arg ($arg_builder:expr) (description => $description:expr) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.description($description)) $($tt)*
        }
    };

    // Argument valid values
    // clapi::app! { (@arg => (values => ... ) }
    (@arg ($arg_builder:expr) (values => $($valid_values:expr),+) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.valid_values(&[$($valid_values),+])) $($tt)*
        }
    };

    // Argument default values
    // clapi::app! { (@arg => (default => ... ) }
    (@arg ($arg_builder:expr) (default => $($default_values:expr),+) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.defaults(&[$($default_values),+])) $($tt)*
        }
    };

    // Argument validator
    // clapi::app! { (@arg => (validator => clapi::validator::parse_validator::<u64>() ) }
    (@arg ($arg_builder:expr) (validator => $validator:expr) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.validator($validator)) $($tt)*
        }
    };

    // Argument type
    // clapi::app! { (@arg => (type => u64 ) }
    (@arg ($arg_builder:expr) (type => $ty:ty) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.validator($crate::validator::parse_validator::<$ty>())) $($tt)*
        }
    };

    // Argument validation error
    // clapi::app! { (@arg => (error => ... ) }
    (@arg ($arg_builder:expr) (error => $error:expr) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.validation_error($error)) $($tt)*
        }
    };

    //////////////////////////////////////////////////////////////////////
    // Some special cases to only create `Command` and not `CommandLine`//
    //////////////////////////////////////////////////////////////////////

    // clapi::app! { @@command => ... }
    (@@command => $($rest:tt)+) => {{
        $crate::app!{
            @command ($crate::Command::root()) $($rest)+
        }
    }};

    // clapi::app! { @@command MyApp => ... }
    (@@command $command_name:ident => $($rest:tt)+) => {{
        $crate::app!{
            @command ($crate::Command::new(stringify!($command_name))) $($rest)+
        }
    }};

    // clapi::app! { @@command "MyApp" => ... }
    (@@command $command_name:expr => $($rest:tt)+) => {{
        $crate::app!{
            @command ($crate::Command::new($command_name)) $($rest)+
        }
    }};
}

/// Constructs and run a `CommandLine` app.
///
/// This is equivalent to:
/// ```ignore
/// clapi::app!(/*...*/)
///     .use_default_suggestions()
///     .use_default_help()
///     .run()
/// ```
#[macro_export]
macro_rules! run_app {
    ( => $($rest:tt)+) => {
        $crate::app!( => $($rest)+)
             .use_default_suggestions()
             .use_default_help()
             .run()
    };

    ($name:ident => $($rest:tt)+) => {
        $crate::app!($name => $($rest)+)
             .use_default_suggestions()
             .use_default_help()
             .run()
    };

    ($name:expr => $($rest:tt)+) => {
        $crate::app!($name => $($rest)+)
             .use_default_suggestions()
             .use_default_help()
             .run()
    };
}

/// Constructs a `CommandLine` app using this crate `Cargo.toml` info.
#[macro_export]
macro_rules! crate_app {
    ($($rest:tt)*) => {{
        $crate::CommandLine::new(
            $crate::app!{
                @command
                (
                    $crate::Command::new($crate::package_name!())
                        .description($crate::package_description!())
                        .subcommand(Command::new("version")
                            .handler(|_, _| {
                                println!("{}", $crate::package_version!());
                                Ok(())
                            })
                        )
                ) $($rest)*
            }
        )
    }};
}

/// Constructs and run a `CommandLine` app using this crate `Cargo.toml` info.
#[macro_export]
macro_rules! run_crate_app {
    ($($rest:tt)*) => {{
        $crate::CommandLine::new(
            $crate::app!{
                @command
                (
                    $crate::Command::new($crate::package_name!())
                        .description($crate::package_description!())
                        .subcommand(Command::new("version")
                            .handler(|_, _| {
                                println!("{}", $crate::package_version!());
                                Ok(())
                            })
                        )
                ) $($rest)*
            }
        )
        .use_default_help()
        .use_default_suggestions()
        .run()
    }};
}