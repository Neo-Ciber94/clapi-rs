use clapi::*;

fn main() {

    // let app = app! { =>
    //     (description => "A command to sum")
    //     (@arg numbers =>
    //         (type => i64)
    //         (count => 0..)
    //     )
    //     (@option times =>
    //         (alias => "t")
    //         (@arg times =>
    //             (type => i64)
    //             (default => 1)
    //         )
    //     )
    //     (handler (times: i64, ...args: Vec<i64>) => {
    //         let sum = args.iter().sum::<i64>();
    //         let total = sum * times;
    //         println!("{:?} * {}, total = {}", args, times, total);
    //     })
    // }.run().unwrap();
    //println!("{:#?}", app);

    let command = Command::new("copy")
        .description("Copies a file from one directory to other")
        .arg(Argument::new("source")
            .description("name of the file"))
        .arg(Argument::new("destination")
            .description("name of the directory"));

    CommandLine::new(command)
        .use_default_suggestions()
        .use_default_help()
        .set_show_help_when_no_handler(true)
        .run()
        .unwrap()
}