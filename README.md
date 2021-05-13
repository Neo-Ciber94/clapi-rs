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

See the examples below creating the same app using the 4 methods.

### Parsing the arguments
```rust
use clapi::{Command, CommandOption, Argument, CommandLine};
use clapi::validator::validate_type;
use std::num::NonZeroUsize;

fn main() -> clapi::Result<()> {
    let command = Command::new("echo")
        .version("1.0")
        .description("outputs the given values on the console")
        .arg(Argument::one_or_more("values"))
        .option(
            CommandOption::new("times")
                .alias("t")
                .description("number of times to repeat")
                .arg(
                    Argument::new()
                        .validator(validate_type::<NonZeroUsize>())
                        .validation_error("expected number greater than 0")
                        .default(NonZeroUsize::new(1).unwrap()),
                ),
        ).handler(|opts, args| {
        let times = opts.convert::<usize>("times").unwrap();
        let values = args.get_raw_args()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(" ") as String;

        for _ in 0..times {
            println!("{}", values);
        }

        Ok(())
    });

    CommandLine::new(command)
        .use_default_help()
        .use_default_suggestions()
        .run()
        .map_err(|e| e.exit())
}
```

### Function handlers
```rust
use clapi::validator::validate_type;
use clapi::{Argument, Command, CommandLine, CommandOption};

fn main() -> clapi::Result<()> {
    let command = Command::new("MyApp")
        .subcommand(
            Command::new("repeat")
                .arg(Argument::one_or_more("values"))
                .option(
                    CommandOption::new("times").alias("t").arg(
                        Argument::with_name("times")
                            .validator(validate_type::<u64>())
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
### Macro
```rust
use std::num::NonZeroUsize;

fn main() -> clapi::Result<()> {
    let cli = clapi::app!{ echo =>
        (version => "1.0")
        (description => "outputs the given values on the console")
        (@option times =>
            (alias => "t")
            (description => "number of times to repeat")
            (@arg =>
                (type => NonZeroUsize)
                (default => NonZeroUsize::new(1).unwrap())
                (error => "expected number greater than 0")
            )
        )
        (@arg values => (count => 1..))
        (handler (times: usize, ...args: Vec<String>) => {
            let values = args.join(" ");
            for _ in 0..times {
                println!("{}", values);
            }
        })
    };

    cli.use_default_suggestions()
        .use_default_help()
        .run()
        .map_err(|e| e.exit())
}
```

### Macro attributes
Requires `macros` feature enable.

```rust
use clapi::macros::*;
use std::num::NonZeroUsize;

#[command(name="echo", description="outputs the given values on the console", version="1.0")]
#[arg(values, min=1)]
#[option(times,
    alias="t",
    description="number of times to repeat",
    default=1,
    error="expected number greater than 0"
)]
fn main(times: NonZeroUsize, values: Vec<String>) {
    let values = values.join(" ");

    for _ in 0..times.get() {
        println!("{}", values);
    }
}
```

## Serde

Any `Command`, `CommandOption` and `Argument` can be serialize/deserialize using the `serde` feature.

This allows you to read or write your command line apps to files.

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