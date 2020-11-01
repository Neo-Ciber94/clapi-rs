use clapi::args::Arguments;
use clapi::command::Command;
use clapi::command_line::CommandLine;
use clapi::error::Result;
use clapi::option::CommandOption;
use clapi::root_command::RootCommand;

extern crate clapi_macros;
use clapi_macros::command;

// #[command(
//     description = "A sample description",
//     help = "A sample help",
// )]
// #[option(name = "x", alias = "number", description = "A number", default = 0)]
// #[option(
//     name = "y",
//     alias = "text",
//     description = "A text",
//     default = "Default text"
// )]
// #[option(
//     name = "z",
//     alias = "true or false",
//     description = "A bool",
//     default = false
// )]
// #[arg(name = "values", default = "one", "two", "tree")]
// fn main(x: u32, y: String, z: bool, values: Vec<String>) {
//     // #[subcommand]
//     // fn other(a: String){
//     //     println!("{}", a);
//     // }
//
//     println!("{}, {}, {}, {:?}", x, y, z, values);
//
// }

fn main() {
    let command = clapi::root_command::RootCommand::new()
        .set_description("A sample help")
        .set_args(
            clapi::args::Arguments::new(clapi::arg_count::ArgCount::new(0, 18446744073709551615))
                .set_default_values(&["one", "two", "tree"]),
        )
        .set_option(
            clapi::option::CommandOption::new("x")
                .set_alias("number")
                .set_description("A number")
                .set_args(
                    clapi::args::Arguments::new(clapi::arg_count::ArgCount::new(
                        0,
                        18446744073709551615,
                    ))
                    .set_default_values(&["0"]),
                ),
        )
        .set_option(
            clapi::option::CommandOption::new("y")
                .set_alias("text")
                .set_description("A text")
                .set_args(
                    clapi::args::Arguments::new(clapi::arg_count::ArgCount::new(
                        0,
                        18446744073709551615,
                    ))
                    .set_default_values(&["Default text"]),
                ),
        )
        .set_option(
            clapi::option::CommandOption::new("z")
                .set_alias("true or false")
                .set_description("A bool")
                .set_args(
                    clapi::args::Arguments::new(clapi::arg_count::ArgCount::new(
                        0,
                        18446744073709551615,
                    ))
                    .set_default_values(&["false"]),
                ),
        )
        .set_handler(|opts, args| {
            println!("values: {:?}", args.values());

            let values = args.convert_all::<String>().unwrap();
            let x = opts.get_args("x").unwrap().convert_at::<u32>(0).unwrap();
            let y = opts.get_args("y").unwrap().convert_at::<String>(0).unwrap();
            let z = opts.get_args("z").unwrap().convert_at::<bool>(0).unwrap();
            println!("{}, {}, {}, {:?}", x, y, z, values);
            Ok(())
        });
    clapi::command_line::CommandLine::new(command)
        .use_default_help()
        .use_default_suggestions()
        .run()
        .expect("an error occurred");
}

#[allow(dead_code)]
fn run_cmd() -> Result<()> {
    let root = RootCommand::new()
        .set_description("A file manager system")
        .set_help("A file manager system for create, find and list files in a directory.")
        .set_option(
            CommandOption::new("version")
                .set_alias("v")
                .set_args(Arguments::new(1).set_name("format"))
                .set_description("Version of the app"),
        )
        .set_option(
            CommandOption::new("author")
                .set_alias("a")
                .set_description("Author of the app"),
        )
        .set_command(
            Command::new("create")
                .set_description("Create a file")
                .set_args(Arguments::new(1))
                .set_handler(|_, args| {
                    println!("Create a file named: {}", args.values()[0]);
                    Ok(())
                }),
        )
        .set_command(
            Command::new("list")
                .set_description("List the files in a directory")
                .set_option(
                    CommandOption::new("sort").set_alias("s").set_args(
                        Arguments::new(1)
                            .set_valid_values(&["date", "size", "name"])
                            .set_default_values(&["name"]),
                    ),
                )
                .set_handler(|opts, _| {
                    println!(
                        "Lists the files by: {:?}",
                        opts.get("s").map(|s| s.args().values())
                    );
                    Ok(())
                }),
        );

    CommandLine::default_with_root(root).run()
}
