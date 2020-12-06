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

// #[command(version=1.0)]
// #[option(value, description="The value", default="Hello World")]
// fn main(value: String) -> clapi::Result<()> {
//     println!("{:?}", value);
//     Ok(())
// }


fn main(){
    let mut cli = debug_app! { =>
        (@arg from)
        (@arg to)
        (@option times =>
            (@arg times =>
                (count => 1..)
                (type => i64)
            )
        )
        (@option enable =>
            (@arg enable)
        )
        (handler (times: Option<i64>, ...from: String, to: String) => {
            println!("{:?}, from: {}, to: {}", times, from, to);
        })
    };
    cli.run().unwrap();
}