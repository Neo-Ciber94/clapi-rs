use clapi::macros::*;

#[command]
#[option(r#type)]
#[arg(r#values)]
fn main(r#type: Option<String>, r#values: Vec<String>) {}
