
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
/// - `@subcommand` : description, about, handler, @subcommand, @option and @arg.
/// - `@option` : description, alias, required and @arg.
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
///     (about => "MyApp 1.0")
///     (@arg values =>
///         (count => 1..)
///         (type => i64)
///     )
///     (@option times =>
///         (description => "Number of times to sum the values")
///         (@arg times =>
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
    // Here start
    (=> $($rest:tt)+) => {{
        $crate::CommandLine::new(
            $crate::app!{
                @command ($crate::Command::root()) $($rest)+
            }
        )
    }};

    ($command_name:ident => $($rest:tt)+) => {{
        $crate::CommandLine::new(
            $crate::app!{
                @command ($crate::Command::new(stringify!($command_name))) $($rest)+
            }
        )
    }};

    ($command_name:expr => $($rest:tt)+) => {{
        $crate::CommandLine::new(
            $crate::app!{
                @command ($crate::Command::new($command_name)) $($rest)+
            }
        )
    }};

    // Special case, to just create a `Command` without the `CommandLine`
    // (@command => $($rest:tt)+) => {{
    //     $crate::app!{
    //         @command ($crate::Command::root()) $($rest)+
    //     }
    // }};
    //
    // (@command $command_name:ident => $($rest:tt)+) => {{
    //     $crate::app!{
    //         @command ($crate::Command::new(stringify!($command_name))) $($rest)+
    //     }
    // }};
    //
    // (@command $command_name:expr => $($rest:tt)+) => {{
    //     $crate::app!{
    //         @command ($crate::Command::new($command_name)) $($rest)+
    //     }
    // }};

    // Command
    (@command ($builder:expr)) => { $builder };

    (@command ($builder:expr) (description => $description:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.description($description)) $($tt)*
        }
    };

    (@command ($builder:expr) (about => $about:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.about($about)) $($tt)*
        }
    };

    // Handler
    (@command ($builder:expr) (handler ($options:ident, $arguments:ident) => $block:block) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|$options, $arguments|{
                $block
                Ok(())
            })) $($tt)*
        }
    };

    (@command ($builder:expr) (handler ($options:ident, $arguments:ident) => $expr:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|$options, $arguments|{
                $expr;
                Ok(())
            })) $($tt)*
        }
    };

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

    (@command ($builder:expr) (handler ($($name:ident : $ty:ty)+ $(,...$($arg_name:ident: $arg_type:ty),+)?) => $block:block) $($tt:tt)*) => {
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

    (@command ($builder:expr) (handler ($($name:ident : $ty:ty)+ $(,...$($arg_name:ident: $arg_type:ty),+)?) => $expr:expr) $($tt:tt)*) => {
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

    (@command ($builder:expr) (handler () => $block:block) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|_options, _arguments|{
                $block
                Ok(())
            })) $($tt)*
        }
    };

    (@command ($builder:expr) (handler () => $expr:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|_options, _arguments|{
                $expr;
                Ok(())
            })) $($tt)*
        }
    };

    (@command ($builder:expr) (handler => $block:block) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|_options, _arguments|{
                $block
                Ok(())
            })) $($tt)*
        }
    };

    (@command ($builder:expr) (handler => $expr:expr) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|_options, _arguments|{
                $expr;
                Ok(())
            })) $($tt)*
        }
    };

    // Subcommand
    (@command ($builder:expr) (@subcommand $command_name:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.subcommand(
                $crate::app!{ @command ($crate::Command::new(stringify!($command_name))) $($($rest)+)? }
            )) $($tt)*
        }
    };

    (@command ($builder:expr) (@subcommand $command_name:expr $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.subcommand(
                $crate::app!{ @command ($crate::Command::new($command_name)) $($($rest)+)? }
            )) $($tt)*
        }
    };

    // Option
    (@command ($builder:expr) (@option $option_name:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.option(
                $crate::app!{ @option ($crate::CommandOption::new(stringify!($option_name))) $($($rest)+)? }
            )) $($tt)*
        }
    };

    (@command ($builder:expr) (@option $option_name:expr $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.option(
                $crate::app!{ @option ($crate::CommandOption::new($option_name)) $($($rest)+)? }
            )) $($tt)*
        }
    };

    (@option ($option_builder:expr)) => { $option_builder };

    (@option ($option_builder:expr) (@arg $arg_name:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @option
            ($option_builder.arg(
                $crate::app!{ @arg ($crate::Argument::new(stringify!($arg_name))) $($($rest)+)? }
            )) $($tt)*
        }
    };

    (@option ($option_builder:expr) (@arg $arg_name:expr $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @option
            ($option_builder.arg(
                $crate::app!{ @arg ($crate::Argument::new($arg_name)) $($($rest)+)? }
            )) $($tt)*
        }
    };

    (@option ($option_builder:expr) (description => $description:expr) $($tt:tt)*) => {
        $crate::app!{
            @option ($option_builder.description($description)) $($tt)*
        }
    };

    (@option ($option_builder:expr) (required => $required:expr) $($tt:tt)*) => {
        $crate::app!{
            @option ($option_builder.required($required)) $($tt)*
        }
    };

    (@option ($option_builder:expr) (alias => $($alias:expr),+) $($tt:tt)*) => {
        $crate::app!{
            @option
            ($option_builder$(.alias($alias))+) $($tt)*
        }
    };

    // Argument
    (@command ($builder:expr) (@arg $arg_name:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.arg(
                $crate::app!{ @arg ($crate::Argument::new(stringify!($arg_name))) $($($rest)+)? }
            )) $($tt)*
        }
    };

    (@command ($builder:expr) (@arg $arg_name:expr $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.arg(
                $crate::app!{ @arg ($crate::Argument::new($arg_name:expr)) $($($rest)+)? }
            )) $($tt)*
        }
    };

    (@arg ($arg_builder:expr)) => { $arg_builder };

    (@arg ($arg_builder:expr) (count => $count:expr) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.arg_count($count)) $($tt)*
        }
    };

    (@arg ($arg_builder:expr) (description => $description:expr) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.description($description)) $($tt)*
        }
    };

    (@arg ($arg_builder:expr) (values => $($valid_values:expr),+) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.valid_values(&[$($valid_values),+])) $($tt)*
        }
    };

    (@arg ($arg_builder:expr) (default => $($default_values:expr),+) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.defaults(&[$($default_values),+])) $($tt)*
        }
    };

    (@arg ($arg_builder:expr) (validator => $validator:expr) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.validator($validator)) $($tt)*
        }
    };

    (@arg ($arg_builder:expr) (type => $ty:ty) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.validator($crate::validator::parse_validator::<$ty>())) $($tt)*
        }
    };
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