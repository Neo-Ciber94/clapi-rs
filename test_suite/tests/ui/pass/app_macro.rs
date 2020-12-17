
fn main(){
    let _app = clapi::app! { MyApp =>
        (description => "Sums a set of numbers")
        (about => "sum 1.0")
        (@option times =>
            (alias => "t", "T")
            (description => "Number of times to sum the numbers")
            (required => false)
            (@arg times =>
                (description => "Number of times")
                (count => 1)
                (type => u64)
            )
        )
        (@arg values =>
            (description => "Numbers to sum")
            (count => 1..)
            (type => i64)
        )
        (@subcommand version =>
            (description => "Shows the version of the app")
            (handler => println!("echo 1.0"))
        )
        (handler (times: u64, ...values: Vec<i64>) => {
            let times = times as i64;
            let total = values.iter().sum::<i64>() * times;
            println!("{}", total);
        })
    };
}