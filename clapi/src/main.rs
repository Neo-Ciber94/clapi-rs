use clapi::*;
use clapi::validator::parse_validator;

// assert_args, assert_non_duplicated_arguments
fn main() {
    let mut cli = debug_app! { =>
        (description => "A command to sum")
        (@option times =>
            (alias => "t")
            (@arg times =>
                (type => i64)
                (default => 1)
            )
        )
        (@arg numbers =>
            (type => i64)
            (count => 1..)
        )
        (handler (times: i32, ...args: Vec<i64>) => {
            let sum = args.iter().sum::<i64>();
            let total = sum * times as i64;
            println!("{:?} * {}, total = {}", args, times, total);
        })
    };
    cli.run().unwrap()
}