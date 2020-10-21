use clapi::root_command::RootCommand;
use clapi::args::Arguments;
use clapi::error::Result;
use clapi::command_line::CommandLine;
use clapi::command::Command;
use clapi::option::CommandOption;

// extern crate clapi_macros;
// use clapi_macros::command;

// #[command]
// fn main(x: u32, y: String, z: bool){
//     println!("Adios mundo");
// }

fn main() -> Result<()>{
    run_cmd()
}

#[allow(dead_code)]
fn run_cmd() -> Result<()>{
    let root = RootCommand::new()
        .set_description("A file manager system")
        .set_help("A file manager system for create, find and list files in a directory.")
        .set_option(CommandOption::new("version")
            .set_alias("v")
            .set_args(Arguments::new(1).set_name("format"))
            .set_description("Version of the app"))
        .set_option(CommandOption::new("author")
            .set_alias("a")
            .set_description("Author of the app"))
        .set_command(Command::new("create")
            .set_description("Create a file")
            .set_args(Arguments::new(1))
            .set_handler(|_, args| {
                println!("Create a file named: {}", args[0]);
                Ok(())
            }))
        .set_command(Command::new("list")
            .set_description("List the files in a directory")
            .set_option(CommandOption::new("sort")
                .set_alias("s")
                .set_args(Arguments::new(1)
                    .set_valid_values(&["date", "size", "name"])
                    .set_default_values(&["name"])))
            .set_handler(|opts, _|{
                println!("Lists the files by: {:?}", opts.get("s")
                    .map(|s| s.args().values()));
                Ok(())
            }));

    CommandLine::default_with_root(root).run()
}

/*
let options = Options::new();
let value = options.get("id").arg_as<u32>();
let values = options.get("numbers").args_as<32>();
*/

/*
#[command(name="cmd", description="A command"]
#[option(path="The path of the file")]
#[option(count="Number of copies")]
fn main(path: String, count: u32) -> Result<()>{
}
*/