use std::num::NonZeroUsize;
use clapi::help::command_help;

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