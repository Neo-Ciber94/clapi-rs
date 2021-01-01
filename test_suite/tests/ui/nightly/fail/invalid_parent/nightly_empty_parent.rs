use clapi::macros::*;

#[command]
fn files(){}

#[subcommand]
fn list(){}

#[subcommand(parent="")]
fn sort(){}

fn main(){}