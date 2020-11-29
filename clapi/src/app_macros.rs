
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
                fn assert_args($($name: $ty),+ $(,$args_name: Vec<$args_type>)?){}

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
                fn assert_args($($name: $ty),+ $(,$args_name: &[$args_type])?){}

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
                fn assert_args($($name: $ty),+ $(,$args_name: $args_type)?){}

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