
/// Creates a new `CommandLine` app.
#[macro_export]
macro_rules! app {
    // Here start
    (=> $($rest:tt)+) => {{
        $crate::CommandLine::new(
            $crate::app!{
                @command ($crate::Command::root()) $($rest)+
            }
        )
        .use_default_help()
        .use_default_suggestions()
    }};

    ($command_name:ident => $($rest:tt)+) => {{
        $crate::CommandLine::new(
            $crate::app!{
                @command ($crate::Command::new(stringify!($command_name))) $($rest)+
            }
        )
        .use_default_help()
        .use_default_suggestions()
    }};

    ($command_name:expr => $($rest:tt)+) => {{
        $crate::CommandLine::new(
            $crate::app!{
                @command ($crate::Command::new($command_name)) $($rest)+
            }
        )
        .use_default_help()
        .use_default_suggestions()
    }};

    // Command
    (@command ($builder:expr)) => { $builder };

    (@command ($builder:expr) (description => $description:literal) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.description($description)) $($tt)*
        }
    };

    (@command ($builder:expr) (about => $about:literal) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.about($about)) $($tt)*
        }
    };

    // Handler
    (@command ($builder:expr) (handler (...$name:ident: Vec<$args_type:ty>) => $block:block ) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|opts, args|{
                let $name = $crate::try_parse_values::<$args_type>(args.get_raw_args())?;
                $block
                Ok(())
            })) $($tt)*
        }
    };

    (@command ($builder:expr) (handler (...$name:ident: &[$ty:ty]) => $block:block ) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|opts, args|{
                let temp = $crate::try_parse_values::<$ty>(args.get_raw_args())?;
                let $name = temp.as_slice();
                $block
                Ok(())
            })) $($tt)*
        }
    };

    (@command ($builder:expr) (handler (...$name:ident: $ty:ty) => $block:block ) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|opts, args|{
                let $name = match $crate::try_parse_values::<$ty>(args.get_raw_args()){
                    Err(error) => { Err(error) },
                    Ok(x) if x.len() == 1 => { Ok(x[0]) },
                    Ok(x) => {
                        Err($crate::Error::new(
                                $crate::ErrorKind::InvalidArgumentCount,
                                format!("`{}` expect 1 value but was {}", stringify!($name), x.len())
                            )
                        )
                    },
                }?;
                $block
                Ok(())
            })) $($tt)*
        }
    };

    (@command ($builder:expr) (handler ($($name:ident: $ty:ty),+ $(,...$args_name:ident: Vec<$args_type:ty>)?) => $block:block ) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|opts, args|{
                #[cfg(debug_assertions)]
                fn assert_non_duplicate_arguments($($name: $ty),+ $(,$args_name: Vec<$args_type>)?){}

                $(
                    let $name = opts.get_arg(stringify!($name)).unwrap().convert::<$ty>()?;
                )+

                $(
                    let $args_name = $crate::try_parse_values::<$args_type>(args.get_raw_args())?;
                )?

                $block
                Ok(())
            })) $($tt)*
        }
    };

    (@command ($builder:expr) (handler ($($name:ident: $ty:ty),+ $(,...$args_name:ident: &[$args_type:ty])?) => $block:block ) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|opts, args|{
                #[cfg(debug_assertions)]
                fn assert_non_duplicate_arguments($($name: $ty),+ $(,$args_name: &[$args_type])?){}

                $(
                    let $name = opts.get_arg(stringify!($name)).unwrap().convert::<$ty>()?;
                )+

                $(
                    let temp = $crate::try_parse_values::<$args_type>(args.get_raw_args())?;
                    let $args_name = temp.as_slice();
                )?

                $block
                Ok(())
            })) $($tt)*
        }
    };

    (@command ($builder:expr) (handler ($($name:ident: $ty:ty),+ $(,...$args_name:ident: $args_type:ty)?) => $block:block ) $($tt:tt)*) => {
        $crate::app!{
            @command ($builder.handler(|opts, args|{
                #[cfg(debug_assertions)]
                fn assert_non_duplicate_arguments($($name: $ty),+ $(,$args_name: $args_type)?){}

                $(
                    let $name = opts.get_arg(stringify!($name)).unwrap().convert::<$ty>()?;
                )+

                $(
                    let $args_name = match $crate::try_parse_values::<$args_type>(args.get_raw_args()){
                        Err(error) => { Err(error) },
                        Ok(x) if x.len() == 1 => { Ok(x[0]) },
                        Ok(x) => {
                            Err($crate::Error::new(
                                    $crate::ErrorKind::InvalidArgumentCount,
                                format!("`{}` expect 1 value but was {}", stringify!($args_name), x.len())
                                )
                            )
                        },
                    }?;
                )?

                $block
                Ok(())
            })) $($tt)*
        }
    };

    // Subcommand
    (@command ($builder:expr) (@subcommand $command:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::app!{
            @command
            ($builder.subcommand(
                $crate::app!{ @command ($crate::Command::new(stringify!($command))) $($($rest)+)? }
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

    (@option ($option_builder:expr) (description => $description:literal) $($tt:tt)*) => {
        $crate::app!{
            @option ($option_builder.description($description)) $($tt)*
        }
    };

    (@option ($option_builder:expr) (required => true) $($tt:tt)*) => {
        $crate::app!{
            @option ($option_builder.required(true)) $($tt)*
        }
    };

    (@option ($option_builder:expr) (required => false) $($tt:tt)*) => {
        $crate::app!{
            @option ($option_builder.required(false)) $($tt)*
        }
    };

    (@option ($option_builder:expr) (alias => $($alias:literal),+) $($tt:tt)*) => {
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

    (@arg ($arg_builder:expr) (description => $description:literal) $($tt:tt)*) => {
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

    (@arg ($arg_builder:expr) (type => $ty:ty) $($tt:tt)*) => {
        $crate::app!{
            @arg ($arg_builder.validator($crate::validator::parse_validator::<$ty>())) $($tt)*
        }
    };
}

#[macro_export]
macro_rules! run_app {
    ( => $($rest:tt)+) => {
        $crate::app!( => $($rest)+).run()
    };

    ($name:ident => $($rest:tt)+) => {
        $crate::app!($name => $($rest)+).run()
    };

    ($name:expr => $($rest:tt)+) => {
        $crate::app!($name => $($rest)+).run()
    };

    ($name:literal => $($rest:tt)+) => {
        $crate::app!($name => $($rest)+).run()
    };
}

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
        .use_default_help()
        .use_default_suggestions()
    }};
}

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

#[macro_export]
macro_rules! crate_name {
    () => {
        option_env!("CARGO_PKG_NAME")
            .expect("package name is not defined")
    };
}

#[macro_export]
macro_rules! crate_description {
    () => {
        option_env!("CARGO_PKG_DESCRIPTION")
            .expect("package description is not defined")
    };
}

#[macro_export]
macro_rules! crate_version {
    () => {
        option_env!("CARGO_PKG_VERSION")
            .expect("package version is not defined")
    };
}

////
#[macro_export]
macro_rules! debug_app {
    // Here start
    (=> $($rest:tt)+) => {{
        let mut type_checker = $crate::type_checker::CommandArgumentTypeChecker::new();

        $crate::CommandLine::new({
            let mut command = $crate::Command::root();
            let command_name = command.get_name().to_owned();
            command = $crate::debug_app!{ @command (command, command_name, type_checker) $($rest)+ };
            command
        })
        .use_default_help()
        .use_default_suggestions()
    }};

    ($command_name:ident => $($rest:tt)+) => {{
        let mut type_checker = $crate::type_checker::CommandArgumentTypeChecker::new();

        $crate::CommandLine::new({
            let mut command = $crate::Command::new(stringify!($command_name));
            let command_name = command.get_name().to_owned();
            command = $crate::debug_app!{ @command (command, command_name, type_checker) $($rest)+ };
            command
        })
        .use_default_help()
        .use_default_suggestions()
    }};

    ($command_name:expr => $($rest:tt)+) => {{
        let mut type_checker = $crate::type_checker::CommandArgumentTypeChecker::new();

        $crate::CommandLine::new({
            let mut command = $crate::Command::new($command_name);
            let command_name = command.get_name().to_owned();
            command = $crate::debug_app!{ @command (command, command_name, type_checker) $($rest)+ };
            command
        })
        .use_default_help()
        .use_default_suggestions()
    }};

    // Command
    (@command ($builder:expr, $command_name:ident, $type_checker:ident)) => { $builder };

    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (description => $description:literal) $($tt:tt)*) => {
        $crate::debug_app!{
            @command ($builder.description($description), $command_name, $type_checker) $($tt)*
        }
    };

    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (about => $about:literal) $($tt:tt)*) => {
        $crate::debug_app!{
            @command ($builder.about($about), $command_name, $type_checker) $($tt)*
        }
    };

    // Handler
    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (handler (...$name:ident: Vec<$args_type:ty>) => $block:block ) $($tt:tt)*) => {
        $crate::debug_app!{
            @command ($builder.handler({
            $type_checker.assert_same_type::<$ty>(&$command_name, stringify!($name));
            |opts, args|{
                let $name = $crate::try_parse_values::<$args_type>(args.get_raw_args())?;
                $block
                Ok(())
            }
            }), $command_name, $type_checker) $($tt)*
        }
    };

    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (handler (...$name:ident: &[$ty:ty]) => $block:block ) $($tt:tt)*) => {
        $crate::debug_app!{
            @command ($builder.handler({
            $type_checker.assert_same_type::<$ty>(&$command_name, stringify!($name));
            |opts, args|{
                let temp = $crate::try_parse_values::<$ty>(args.get_raw_args())?;
                let $name = temp.as_slice();
                $block
                Ok(())
            }
            }), $command_name, $type_checker) $($tt)*
        }
    };

    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (handler (...$name:ident: $ty:ty) => $block:block ) $($tt:tt)*) => {
        $crate::debug_app!{
            @command ($builder.handler({
            $type_checker.assert_same_type::<$ty>(&$command_name, stringify!($name));
            |opts, args|{
                let $name = match $crate::try_parse_values::<$ty>(args.get_raw_args()){
                    Err(error) => { Err(error) },
                    Ok(x) if x.len() == 1 => { Ok(x[0]) },
                    Ok(x) => {
                        Err($crate::Error::new(
                                $crate::ErrorKind::InvalidArgumentCount,
                                format!("`{}` expect 1 value but was {}", stringify!($name), x.len())
                            )
                        )
                    },
                }?;
                $block
                Ok(())
            }
            }), $command_name, $type_checker) $($tt)*
        }
    };

    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (handler ($($name:ident: $ty:ty),+ $(,...$args_name:ident: Vec<$args_type:ty>)?) => $block:block ) $($tt:tt)*) => {
        $crate::debug_app!{
            @command ($builder.handler({
                $($type_checker.assert_same_type::<$ty>(&$command_name, stringify!($name));)+
                $($type_checker.assert_same_type::<$args_type>(&$command_name, stringify!($args_name));)?
                |opts, args|{
                    #[cfg(debug_assertions)]
                    fn assert_non_duplicate_arguments($($name: $ty),+ $(,$args_name: Vec<$args_type>)?){}

                    $(
                        let $name = opts.get_arg(stringify!($name)).unwrap().convert::<$ty>()?;
                    )+
                    $(
                        let $args_name = $crate::try_parse_values::<$args_type>(args.get_raw_args())?;
                    )?

                    $block
                    Ok(())
                }
            }), $command_name, $type_checker) $($tt)*
        }
    };

    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (handler ($($name:ident: $ty:ty),+ $(,...$args_name:ident: &[$args_type:ty])?) => $block:block ) $($tt:tt)*) => {
        $crate::debug_app!{
            @command ($builder.handler({
                $($type_checker.assert_same_type::<$ty>(&$command_name, stringify!($name));)+
                $($type_checker.assert_same_type::<$args_type>(&$command_name, stringify!($args_name));)?
                |opts, args|{
                    #[cfg(debug_assertions)]
                    fn assert_non_duplicate_arguments($($name: $ty),+ $(,$args_name: &[$args_type])?){}

                    $(
                        let $name = opts.get_arg(stringify!($name)).unwrap().convert::<$ty>()?;
                    )+
                    $(
                        let temp = $crate::try_parse_values::<$args_type>(args.get_raw_args())?;
                        let $args_name = temp.as_slice();
                    )?

                    $block
                    Ok(())
                }
            }), $command_name, $type_checker) $($tt)*
        }
    };

    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (handler ($($name:ident: $ty:ty),+ $(,...$args_name:ident: $args_type:ty)?) => $block:block ) $($tt:tt)*) => {
        $crate::debug_app!{
            @command ($builder.handler({
                $($type_checker.assert_same_type::<$ty>(&$command_name, stringify!($name));)+
                $($type_checker.assert_same_type::<$args_type>(&$command_name, stringify!($args_name));)?
                |opts, args|{
                    #[cfg(debug_assertions)]
                    fn assert_non_duplicate_arguments($($name: $ty),+ $(,$args_name: $args_type)?){}

                    $(
                        let $name = opts.get_arg(stringify!($name)).unwrap().convert::<$ty>()?;
                    )+
                    $(
                        let $args_name = match $crate::try_parse_values::<$args_type>(args.get_raw_args()){
                            Err(error) => { Err(error) },
                            Ok(x) if x.len() == 1 => { Ok(x[0]) },
                            Ok(x) => {
                                Err($crate::Error::new(
                                        $crate::ErrorKind::InvalidArgumentCount,
                                    format!("`{}` expect 1 value but was {}", stringify!($args_name), x.len())
                                    )
                                )
                            },
                        }?;
                    )?

                    $block
                    Ok(())
                }
            }), $command_name, $type_checker) $($tt)*
        }
    };

    // Subcommand
    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (@subcommand $command:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::debug_app!{
            @command
            ($builder.subcommand({
                let mut command = $crate::Command::new(stringify!($command));
                let command_name = command.get_name().to_owned();
                command = $crate::debug_app!{ @command (command, $command_name, $type_checker) $($($rest)+)? };
                command
            }), $command_name, $type_checker) $($tt)*
        }
    };

    // Option
    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (@option $option_name:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::debug_app!{
            @command
            ($builder.option(
                $crate::debug_app!{ @option ($crate::CommandOption::new(stringify!($option_name)), $command_name, $type_checker) $($($rest)+)? }
            ), $command_name, $type_checker) $($tt)*
        }
    };

    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (@option $option_name:expr $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::debug_app!{
            @command
            ($builder.option(
                $crate::debug_app!{ @option ($crate::CommandOption::new($option_name), $command_name, $type_checker) $($($rest)+)? }
            ), $command_name, $type_checker) $($tt)*
        }
    };

    (@option ($option_builder:expr, $command_name:ident, $type_checker:ident)) => { $option_builder };

    (@option ($option_builder:expr, $command_name:ident, $type_checker:ident) (@arg $arg_name:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::debug_app!{
            @option
            ($option_builder.arg({
                let mut arg = $crate::Argument::new(stringify!($arg_name));
                let arg_name = arg.get_name().to_owned();
                arg = $crate::debug_app!{ @arg (arg, $command_name, arg_name, $type_checker) $($($rest)+)? };
                arg
            }), $command_name, $type_checker) $($tt)*
        }
    };

    (@option ($option_builder:expr, $command_name:ident, $type_checker:ident) (@arg $arg_name:expr $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::debug_app!{
            @option
            ($option_builder.arg({
                let mut arg = $crate::Argument::new($arg_name);
                let arg_name = arg.get_name().to_owned();
                arg = $crate::debug_app!{ @arg (arg, $command_name, arg_name, $type_checker) $($($rest)+)? };
                arg
            }), $command_name, $type_checker) $($tt)*
        }
    };

    (@option ($option_builder:expr, $command_name:ident, $type_checker:ident) (description => $description:literal) $($tt:tt)*) => {
        $crate::debug_app!{
            @option ($option_builder.description($description), $command_name, $type_checker) $($tt)*
        }
    };

    (@option ($option_builder:expr, $command_name:ident, $type_checker:ident) (required => true) $($tt:tt)*) => {
        $crate::debug_app!{
            @option ($option_builder.required(true), $command_name, $type_checker) $($tt)*
        }
    };

    (@option ($option_builder:expr, $command_name:ident, $type_checker:ident) (required => false) $($tt:tt)*) => {
        $crate::debug_app!{
            @option ($option_builder.required(false), $command_name, $type_checker) $($tt)*
        }
    };

    (@option ($option_builder:expr, $command_name:ident, $type_checker:ident) (alias => $($alias:literal),+) $($tt:tt)*) => {
        $crate::debug_app!{
            @option
            ($option_builder$(.alias($alias))+, $command_name, $type_checker) $($tt)*
        }
    };

    // Argument
    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (@arg $arg_name:ident $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::debug_app!{
            @command
            ($builder.arg({
                let mut arg = $crate::Argument::new(stringify!($arg_name));
                let arg_name = arg.get_name().to_owned();
                arg = $crate::debug_app!{ @arg (arg, $command_name, arg_name, $type_checker) $($($rest)+)? };
                arg
            }), $command_name, $type_checker) $($tt)*
        }
    };

    (@command ($builder:expr, $command_name:ident, $type_checker:ident) (@arg $arg_name:expr $(=> $($rest:tt)+)?) $($tt:tt)*) => {
        $crate::debug_app!{
            @command
            ($builder.arg({
                let mut arg = $crate::Argument::new($arg_name:expr);
                let arg_name = arg.get_name().to_owned();
                arg = $crate::debug_app!{ @arg (arg, $command_name, arg_name, $type_checker) $($($rest)+)? };
                arg
            }), $command_name, $type_checker) $($tt)*
        }
    };

    (@arg ($arg_builder:expr, $command_name:ident, $arg_name:ident, $type_checker:ident)) => { $arg_builder };

    (@arg ($arg_builder:expr, $command_name:ident, $arg_name:ident, $type_checker:ident) (count => $count:expr) $($tt:tt)*) => {
        $crate::debug_app!{
            @arg ($arg_builder.arg_count($count), $command_name, $arg_name, $type_checker) $($tt)*
        }
    };

    (@arg ($arg_builder:expr, $command_name:ident, $arg_name:ident, $type_checker:ident) (description => $description:literal) $($tt:tt)*) => {
        $crate::debug_app!{
            @arg ($arg_builder.description($description), $command_name, $arg_name, $type_checker) $($tt)*
        }
    };

    (@arg ($arg_builder:expr, $command_name:ident, $arg_name:ident, $type_checker:ident) (values => $($valid_values:expr),+) $($tt:tt)*) => {
        $crate::debug_app!{
            @arg ($arg_builder.valid_values(&[$($valid_values),+]), $command_name, $arg_name, $type_checker) $($tt)*
        }
    };

    (@arg ($arg_builder:expr, $command_name:ident, $arg_name:ident, $type_checker:ident) (default => $($default_values:expr),+) $($tt:tt)*) => {
        $crate::debug_app!{
            @arg ($arg_builder.defaults(&[$($default_values),+]), $command_name, $arg_name, $type_checker) $($tt)*
        }
    };

    (@arg ($arg_builder:expr, $command_name:ident, $arg_name:ident, $type_checker:ident) (type => $ty:ty) $($tt:tt)*) => {{
        $crate::debug_app!{
            @arg ($arg_builder.validator({
                let validator = $crate::validator::parse_validator::<$ty>();
                $type_checker.add_argument::<$ty>($command_name.clone(), $arg_name.clone());
                validator
            }), $command_name, $arg_name, $type_checker) $($tt)*
        }
    }};
}

#[macro_export]
macro_rules! debug_run_app {
    ( => $($rest:tt)+) => {
        $crate::debug_run_app!( => $($rest)+).run()
    };

    ($name:ident => $($rest:tt)+) => {
        $crate::debug_run_app!($name => $($rest)+).run()
    };

    ($name:expr => $($rest:tt)+) => {
        $crate::debug_run_app!($name => $($rest)+).run()
    };

    ($name:literal => $($rest:tt)+) => {
        $crate::debug_run_app!($name => $($rest)+).run()
    };
}

#[macro_export]
macro_rules! debug_crate_app {
    ($($rest:tt)*) => {{
        $crate::CommandLine::new(
            $crate::debug_app!{
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
    }};
}

#[macro_export]
macro_rules! debug_run_crate_app {
    ($($rest:tt)*) => {{
        $crate::CommandLine::new(
            $crate::debug_app!{
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

// No public API
#[doc(hidden)]
pub mod type_checker {
    use std::collections::HashMap;
    use std::any::{TypeId, type_name};

    #[derive(Debug, Clone)]
    pub struct CommandArgumentTypeChecker {
        map: HashMap<String, HashMap<String, Type>>,
    }

    impl CommandArgumentTypeChecker {
        pub fn new() -> Self {
            CommandArgumentTypeChecker {
                map: Default::default()
            }
        }

        pub fn add_argument<T: 'static>(&mut self, command_name: String, arg_name: String) {
            if let Some(map) = self.map.get_mut(&command_name) {
                map.insert(arg_name, Type::of::<T>());
            } else {
                let mut inner = HashMap::new();
                inner.insert(arg_name, Type::of::<T>());
                self.map.insert(command_name, inner);
            }
        }

        pub fn assert_same_type<T: 'static>(&self, command_name: &str, arg_name: &str) {
            if let Some(map) = self.map.get(command_name) {
                if let Some(r#type) = map.get(arg_name) {
                    let expected = r#type;
                    let current = &Type::of::<T>();
                    if expected != current {
                        panic!("invalid argument type for `{}`, expected `{}` but was `{}`",
                               arg_name,
                               expected.type_name,
                               current.type_name);
                    }
                }
            }
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Type {
        type_name: String,
        type_id: TypeId,
    }

    impl Type {
        pub fn of<T: 'static>() -> Type {
            let type_name = type_name::<T>().to_owned();
            let type_id = TypeId::of::<T>();
            Type { type_name, type_id }
        }
    }
}