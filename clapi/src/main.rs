use clapi::*;

fn main() {
    //trace_macros!(true);
    let app = app! { =>
        (description => "A command to sum")
        (@arg numbers =>
            (type => i64)
            (count => 0..)
        )
        (@option times =>
            (alias => "t")
            (@arg times =>
                (type => i64)
                (default => 1)
            )
        )
        (handler (times: i64, ...args: Vec<i64>) => {
            let sum = args.iter().sum::<i64>();
            let total = sum * times;
            println!("{:?} * {}, total = {}", args, times, total);
        })
    };
    //trace_macros!(false);

    println!("{:#?}", app);
}