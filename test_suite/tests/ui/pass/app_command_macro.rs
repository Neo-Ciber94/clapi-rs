
fn main(){
    let _app : clapi::Command = clapi::app! { @@command MyApp =>
        (description => "Sums a set of numbers")
        (usage =>
        "
            command [--negate] [--times] <numbers...>
            command author [--count]
            command version
        ")
        (@option times =>
            (alias => "t", "T")
            (description => "Number of times to sum the numbers")
            (required => false)
            (multiple => true)
            (hidden => false)
            (requires_assign => true)
            (@arg =>
                (count => 1)
                (type => u64)
            )
        )
        (@option "negate" =>
            (alias => "n")
            (description => "Negates the result")
        )
        (@arg values =>
            (description => "Numbers to sum")
            (count => 1..)
            (type => i64)
        )
        (@subcommand "author" =>
            (description => "Shows the authors")
            (hidden => false)
            (@arg "count" =>
                (description => "Number of authors to show")
                (type => u64)
                (default => 1)
            )
            (handler (...count: u64) => {
                for _ in 0..count {
                    println!("Witch Echidna");
                }
            })
        )
        (@subcommand version =>
            (description => "Shows the version of the app")
            (handler => println!("echo 1.0"))
        )
        (handler (times: u64, negate: bool, ...values: Vec<i64>) => {
            let times = times as i64;
            let mut total = values.iter().sum::<i64>() * times;
            if negate {
                total *= -1;
            }
            println!("{}", total);
        })
    };
}