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

// #[subcomand(parent="count", description="hello")]
// pub fn reverse(){
//
// }
//
// #[subcommand(parent="reverse")]
// pub fn get(){
//
// }


pub mod internal {
    use super::*;

    #[subcommand]
    pub fn other(){
        println!("Other");
    }
}