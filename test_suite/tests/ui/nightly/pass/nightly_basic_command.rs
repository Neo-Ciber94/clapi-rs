use clapi::macros::*;

#[command(description="Prints a value", usage="How to use the command", version=1.0)]
#[option(times, alias="t", arg="count", default=1, description="Times to repeat")]
#[arg(values, name="text", min=1, max=100, description="Values to write")]
fn echo(times: u64, values: Vec<String>){
}

#[subcommand(description=r#"Shows the author of the command"#, version=2.1)]
fn author(){}

#[subcommand(description="list the authors", parent="author")]
fn list(){}

fn main(){}