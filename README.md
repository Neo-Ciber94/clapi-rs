# Clapi

Clapi (Command-Line API) is a framework for create command line applications.

Currently clapi provides several methods for create command line applications:

- Parsing the arguments
- Function handlers
- Macros
- Macros attributes

## Examples

See the examples below creating the same app using the 4 methods.

<details>
<summary><strong>Parsing the arguments</strong></summary>

```rust
use clapi::{Command, CommandOption, Argument, Parser, Context};
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
let result = Parser.parse(&context, std::env::args().skip(1)).expect("unexpected error");

if result.contains_option("version") {
    println!("MyApp 1.0");
    return;
}

if result.command().get_name() == "repeat" {
    let times = result.get_option_arg("times")
        .unwrap()
        .convert::<u64>()
        .unwrap(); // This is safe because default is 1

    let values = result.arg().unwrap()
        .convert_all::<String>()
        .expect("not values specify")
        .join(" ");

    for _ in 0..times {
        println!("{}", values);
    }
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
                    (count => 1..)
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

// Mark the app entry point with `command` attribute
#[command(version=1.0)]
fn main(){ }

// Mark a function as a `subcommand` and defines if `option` and `arg`
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
