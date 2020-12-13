use clapi::*;

// mod count;
// mod utils;

// #[command]
// #[option(number, arg="n", default=1)]
// fn main(number: i64){
//     println!("{}", number);
// }

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Point {
    x: i64,
    y: i64
}

fn main(){
    let mut app = app! { =>
        (description => "Sum the numbers")
        (@arg numbers =>
            (type => i64)
            (default => 0)
            (count => 1..)
        )
        (handler (...numbers: Vec<f64>) => {
            println!("{}", numbers.iter().sum::<f64>());
        })
    };

    app.run().unwrap();
}