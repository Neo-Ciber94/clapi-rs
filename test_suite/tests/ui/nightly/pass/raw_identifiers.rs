use clapi::macros::*;

#[command]
#[option(type)]
#[args(values)]
fn main(r#type: Option<String>, r#values: Vec<String>) {}
