use clapi::*;

// mod count;
// mod utils;

// #[allow(dead_code)]
// #[subcommand(description="Prints a value to the console")]
// #[option(times, alias="t", default=1)]
// #[arg(values)]
// fn echo(times: usize, values: Vec<String>) {
//     for _ in 0..times {
//         for value in &values {
//             print!("{} ", value);
//         }
//
//         println!();
//     }
// }

#[command(description="Prints a value", about="How to use the command", version=1.0)]
#[option(times, alias="t", arg="count", default=1, description="Times to repeat")]
#[arg(values, arg="text", min=1, max=100, description="Values to write")]
fn echo(times: u64, values: Vec<String>){
    #[subcommand(description=r#"Shows the author of the command"#, version=2.1)]
    fn author(){}
}

fn main(){}

// fn main(enable: bool) -> clapi::Result<()> {
//     println!("enable: {:?}", enable);
//     Ok(())
// }
