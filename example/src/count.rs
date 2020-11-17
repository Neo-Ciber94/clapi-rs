use super::*;

#[subcommand]
#[arg(name="min")]
#[arg(name="max")]
#[option(name="closed", default=true)]
pub fn count(min: usize, max: usize, closed: bool) {
    assert!(min <= max);

    let min = min;
    let max = if closed { max + 1 } else { max };
    let mut iter = (min..max).into_iter().peekable();

    while let Some(i) = iter.next(){
        print!("{}", i);

        if iter.peek().is_some() {
            print!(", ");
        }
    }
}