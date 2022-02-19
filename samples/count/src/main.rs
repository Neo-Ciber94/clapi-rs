use clapi::macros::*;

#[command]
fn main(r#type: Option<String>) {
    println!("{:?}", r#type);
}
