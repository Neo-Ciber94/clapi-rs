use super::*;

#[subcommand]
#[arg(min)]
#[arg(max)]
#[option(closed, default=true)]
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

pub mod internal {
    use super::*;

    #[subcommand(parent="super::count")]
    pub fn print10(){
        println!("Diez = ten = 10");
    }
}