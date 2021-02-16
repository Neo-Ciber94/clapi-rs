#[macro_use]
extern crate bencher;
use bencher::{black_box, Bencher};
use clapi::{Argument, Command, CommandOption, Context};

fn help_with_new_string(b: &mut Bencher) {
    let c = new_command();
    let context = Context::new(c);

    b.iter(|| {
        let mut buf = String::new();
        clapi::help::command_help(&mut buf, &context, context.root(), true);
        black_box(buf)
    })
}

fn help_with_string_capacity_16(b: &mut Bencher) {
    let c = new_command();
    let context = Context::new(c);

    b.iter(|| {
        let mut buf = String::with_capacity(16);
        clapi::help::command_help(&mut buf, &context, context.root(), true);
        black_box(buf)
    })
}

fn help_with_string_capacity_32(b: &mut Bencher) {
    let c = new_command();
    let context = Context::new(c);

    b.iter(|| {
        let mut buf = String::with_capacity(32);
        clapi::help::command_help(&mut buf, &context, context.root(), true);
        black_box(buf)
    })
}

fn help_with_string_capacity_64(b: &mut Bencher) {
    let c = new_command();
    let context = Context::new(c);

    b.iter(|| {
        let mut buf = String::with_capacity(64);
        clapi::help::command_help(&mut buf, &context, context.root(), true);
        black_box(buf)
    })
}

fn help_with_string_capacity_128(b: &mut Bencher) {
    let c = new_command();
    let context = Context::new(c);

    b.iter(|| {
        let mut buf = String::with_capacity(128);
        clapi::help::command_help(&mut buf, &context, context.root(), true);
        black_box(buf)
    })
}

fn new_command() -> Command {
    Command::new("App")
        .description("A sample app")
        .arg(Argument::with_name("value"))
        .option(
            CommandOption::new("color")
                .alias("c")
                .description("The color to use")
                .arg(Argument::new().valid_values(&["red", "green", "blue"])),
        )
        .option(
            CommandOption::new("times")
                .description("Number of times to execute")
                .arg(Argument::new()),
        )
        .subcommand(Command::new("test").description("Test the application"))
        .subcommand(Command::new("beta").description("Use beta features"))
}

benchmark_group!(
    benches,
    help_with_new_string,
    help_with_string_capacity_16,
    help_with_string_capacity_32,
    help_with_string_capacity_64,
    help_with_string_capacity_128
);
benchmark_main!(benches);
