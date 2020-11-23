#![allow(dead_code)]
use clapi::macros::*;

// mod count;
// mod utils;

use clapi::Arguments;
use clapi::Command;
use clapi::CommandLine;
use clapi::Result;
use clapi::CommandOption;
use clapi::RootCommand;

// #[subcommand(description="Prints a value to the console")]
// #[option(name="times", alias="t", default=1)]
// #[arg(name="values")]
fn echo(times: usize, values: Vec<String>) {
    for _ in 0..times {
        for value in &values {
            print!("{} ", value);
        }

        println!();
    }
}

#[command]
#[option(name="repeat", alias="r", default=1)]
#[arg(name="numbers")]
fn main(repeat: usize, numbers: Vec<i32>)  -> Result<()> {
    #[subcommand]
    #[arg(name="min")]
    #[arg(name="max")]
    fn count2(min: i32, max: u32){
        println!("min: {}, max: {}", min, max);
    }

    for _ in 0..repeat {
        println!("numbers: {:?}", numbers);
    }

    Ok(())
}

fn run_cmd() -> Result<()> {
    let root = RootCommand::new()
        .description("A file manager system")
        .help("A file manager system for create, find and list files in a directory.")
        .option(
            CommandOption::new("version")
                .alias("v")
                .arg(Arguments::new(1).name("format"))
                .description("Version of the app"),
        )
        .option(
            CommandOption::new("author")
                .alias("a")
                .description("Author of the app"),
        )
        .subcommand(
            Command::new("create")
                .description("Create a file")
                .args(Arguments::new(1))
                .handler(|_, args| {
                    println!("Create a file named: {}", args.get_values()[0]);
                    Ok(())
                }),
        )
        .subcommand(
            Command::new("list")
                .description("List the files in a directory")
                .option(
                    CommandOption::new("sort").alias("s").arg(
                        Arguments::new(1)
                            .valid_values(&["date", "size", "name"])
                            .default_values(&["name"]),
                    ),
                )
                .handler(|opts, _| {
                    println!(
                        "Lists the files by: {:?}",
                        opts.get("s").map(|s| s.get_args().get_values())
                    );
                    Ok(())
                }),
        );

    CommandLine::new(root)
        .use_default_help()
        .use_default_suggestions()
        .run()
}
