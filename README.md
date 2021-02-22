# Clapi

> Clapi (**C**ommand-**L**ine **API**) a rust framework for create command line applications.

[![Apache-2.0]][license]

[Apache-2.0]: https://img.shields.io/badge/LICENSE-Apache--2.0-blue
[license]: https://github.com/Neo-Ciber94/clapi-rs/blob/master/LICENSE

Currently clapi provides several methods for create command line applications:

- Parsing the arguments
- Function handlers
- Macros
- Macros attributes

> Minimum rust version: 1.48

## Features
- `macros` : Enable the used of macro attributes.
- `serde` : Enable Serialize and Deserialize using the `serde` crate.

## Examples

See the examples below creating the same app using the 4 methods.

<details>
<summary><strong>Parsing the arguments</strong></summary>

```rust
use clapi::{Command, CommandOption, Argument, Parser, Context};
use clapi::help::{DefaultHelp, HelpKind, Help, Buffer};
use clapi::validator::parse_validator;

let command = Command::root()
    .option(CommandOption::new("version").alias("v"))
    .subcommand(Command::new("repeat")
        .arg(Argument::one_or_more("values"))
        .option(CommandOption::new("times").alias("t")
            .arg(Argument::new("times")
                .validator(parse_validator::<u64>())
                .default(1))));

let context = Context::new(command);
let result = Parser::new(&context)
                .parse(std::env::args().skip(1))
                .expect("unexpected error");

if result.contains_option("version") {
    println!("MyApp 1.0");
    return;
}

if result.command().get_name() == "repeat" {
    let times = result.get_option_arg("times")
        .unwrap()
        .convert::<u64>()
        .unwrap();

    let values = result.arg().unwrap()
        .convert_all::<String>()
        .expect("error")
        .join(" ");

    for _ in 0..times {
        println!("{}", values);
    }
} else {
    // Fallthrough
    static HELP : DefaultHelp = DefaultHelp(HelpKind::Any);

    let mut buffer = Buffer::new();
    HELP.help(&mut buffer, &context, result.command()).unwrap();
    println!("{}", buffer);
}
```
</details>


<details>
    <summary>
        <strong>Function handlers</strong>
    </summary>

```rust
use clapi::validator::parse_validator;
use clapi::{Argument, Command, CommandLine, CommandOption};

fn main() -> clapi::Result<()> {
    let command = Command::root()
        .option(CommandOption::new("version").alias("v"))
        .handler(|opts, _args| {
            if opts.contains("version") {
                println!("MyApp 1.0");
            }
            Ok(())
     })
     .subcommand(
         Command::new("repeat")
            .arg(Argument::one_or_more("values"))
            .option(
                CommandOption::new("times").alias("t").arg(
                    Argument::new("times")
                        .validator(parse_validator::<u64>())
                        .default(1),
                ),
            )
            .handler(|opts, args| {
                let times = opts.get_arg("times").unwrap().convert::<u64>()?;
                let values = args
                    .get("values")
                    .unwrap()
                    .convert_all::<String>()?
                    .join(" ");

                for _ in 0..times {
                    println!("{}", values);
                }
                Ok(())
            }),
    );

 CommandLine::new(command)
    .use_default_suggestions()
    .use_default_help()
    .run()
}
```
</details>

<details>
    <summary>
        <strong>Macros</strong>
    </summary>

```rust
fn main() -> clapi::Result<()> {
    let cli = clapi::app!{ =>
        (@option version => (alias => "v"))
        (handler () => println!("MyApp 1.0"))
        (@subcommand repeat =>
            (@arg values => (count => 1..))
            (@option times =>
                (alias => "t")
                (@arg times =>
                    (type => u64)
                    (default => 1)
                    (count => 1)
                )
            )
            (handler (times: u64, ...values: Vec<String>) => {
                let values = values.join(" ");
                for _ in 0..times {
                    println!("{}", values);
                }
            })
        )
    };

    cli.use_default_help()
       .use_default_suggestions()
       .run()
}
```
</details>

<details>
    <summary>
        <strong>Macro attributes</strong>
    </summary>

Requires `macros` feature enable.

```rust
use clapi::macros::*;

#[command(version=1.0)]
fn main(){ }

#[subcommand]
#[option(times, alias="t", default=1)]
#[arg(values, min=1)]
fn repeat(times: u32, values: Vec<String>){
    let values = values.join(" ");
    for _ in 0..times {
        println!("{}", values);
    }
}
```
</details>

## Serde

Any `Command`, `CommandOption` and `Argument` can be serialize/deserialize using the `serde` feature.

This allow you to read or write your command line apps to files.

```rust
use clapi::Command;

fn main(){
    let command = serde_json::from_str::<Command>(r#"
    {
        "name" : "MyApp",
        "description" : "An app to sum numbers",
        "options" : [
            {
                "name" : "times",
                "description" : "Number of times to sum the values",
                "args" : [
                    {
                        "name" : "times",
                        "type" : "u64"
                    }
                ]
            }
        ],
        "args" : [
            {
                "name" : "numbers",
                "description" : "Numbers to sum",
                "type" : "i64",
                "min_count" : 1
            }
        ]
    }
    "#).unwrap();

    assert_eq!(command.get_name(), "MyApp");
    assert_eq!(command.get_description(), Some("An app to sum numbers"));
    assert!(command.get_options().get("times").is_some());
    assert!(command.get_args().get("numbers").is_some());
}
```