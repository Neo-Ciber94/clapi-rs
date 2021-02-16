#[macro_use]
extern crate bencher;
use bencher::{black_box, Bencher};
use clapi::{Command, Context, Parser};

fn empty_command(b: &mut Bencher) {
    b.iter(|| {
        black_box(Command::new("App"));
    })
}

fn empty_root_command(b: &mut Bencher) {
    b.iter(|| {
        black_box(Command::root());
    })
}

fn empty_command_parse_from(b: &mut Bencher) {
    b.iter(|| {
        let command = Command::new("App");
        black_box(command.parse_from(empty_args()).unwrap());
    })
}

fn empty_command_parse_with_parser(b: &mut Bencher) {
    b.iter(|| {
        let command = Command::new("App");
        let context = Context::new(command);
        let mut parser = Parser::new(&context);
        black_box(parser.parse(empty_args()).unwrap());
    })
}

#[inline(always)]
fn empty_args() -> Vec<String> {
    vec![]
}

benchmark_group!(
    benches,
    empty_command,
    empty_root_command,
    empty_command_parse_from,
    empty_command_parse_with_parser
);

benchmark_main!(benches);