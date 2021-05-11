use clapi::macros::*;
use std::num::NonZeroUsize;

#[command(name="echo", description="outputs the given values on the console", version="1.0")]
#[arg(values, min=1)]
#[option(times,
    alias="t",
    description="number of times to repeat",
    default=1,
    error="expected number greater than 0"
)]
fn main(times: NonZeroUsize, values: Vec<String>) -> clapi::Result<()> {
    let values = values.join(" ");

    for _ in 0..times.get() {
        println!("{}", values);
    }

    Ok(())
}