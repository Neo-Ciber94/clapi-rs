use clapi::args::Arguments;
use clapi::command::Command;
use clapi::command_line::CommandLine;
use clapi::error::Result;
use clapi::option::CommandOption;
use clapi::root_command::RootCommand;

extern crate clapi_macros;
use clapi_macros::command;

#[command(
    description = "A sample description",
    help = "A sample help",
    default = 1, 2, 3
)]
#[option(name = "x", alias = "number", description = "A number", default = 0)]
#[option(
    name = "y",
    alias = "text",
    description = "A text",
    default = "Default text"
)]
#[option(
    name = "z",
    alias = "true or false",
    description = "A bool",
    default = false
)]
#[args(name = "values", min = 1, max = 2, default = "one", "two", "tree", id(hello=10))]
fn main(x: u32, y: String, z: bool, values: Vec<String>) {
    fn other(a: String){

    }
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
