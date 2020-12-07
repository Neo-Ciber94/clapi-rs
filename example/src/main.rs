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

#[command(version=1.0)]
fn main(enable: bool) -> clapi::Result<()> {
    println!("enable: {:?}", enable);
    Ok(())
}
