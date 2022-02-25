use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[macro_export]
macro_rules! println_colored {
    ($colored:expr, $($arg:tt)*) => {
        use std::io::Write;
        use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

        let color_choice = if $colored {
            ColorChoice::Always
        } else {
            ColorChoice::Never
        };
        let mut stdout = StandardStream::stdout(color_choice);
        stdout.set_color(ColorSpec::new()
                .set_fg(Some(Color::Yellow))
                .set_intense(true)).unwrap();
        writeln!(&mut stdout, $($arg)*).unwrap();
        stdout.reset().unwrap();
    };
}

#[allow(dead_code)]
pub fn print_colored(s: &str, colored: bool) {
    let color_choice = if colored {
        ColorChoice::Always
    } else {
        ColorChoice::Never
    };

    let mut stdout = StandardStream::stdout(color_choice);
    stdout
        .set_color(
            ColorSpec::new()
                .set_fg(Some(Color::Yellow))
                .set_intense(true),
        )
        .ok();

    writeln!(&mut stdout, "{}", s).ok();
    stdout.reset().ok();
}
