use clapi::*;
//use clapi::help::{HelpProvider, HelpKind};

mod count;
mod utils;

#[command]
fn main(){

}

// #[command]
// fn main() {
//
//     let _command = Command::new("MyApp")
//         .description("An app to sum numbers")
//         .arg(
//             Argument::one_or_more("numbers")
//                 .description("Numbers to sum")
//                 .validator(parse_validator::<i64>()),
//         )
//         .option(
//             CommandOption::new("times")
//                 .description("Times to sum the numbers")
//                 .arg(Argument::new("times").validator(parse_validator::<u64>())),
//         )
//         .option(
//             CommandOption::new("add")
//                 .description("Number to add")
//                 .arg(Argument::new("number").validator(parse_validator::<i64>())),
//         )
//         .option(
//             CommandOption::new("sub")
//                 .description("Number to sub")
//                 .arg(Argument::new("number").validator(parse_validator::<i64>())),
//         )
//         .subcommand(Command::new("version").description("Shows the version of the command"));
//
//
//     println!("{:#?}", Command::root());
//
//     //println!("{}", serde_json::to_string_pretty(&command).unwrap());
//     //println!("{}", serde_yaml::to_string(&command).unwrap());
// }

// #[help]
// static HELP : MyHelp = MyHelp;
//
// struct MyHelp;
// impl HelpProvider for MyHelp {
//     fn help(&self, context: &Context, command: &Command) -> String {
//         unimplemented!()
//     }
//
//     fn usage(&self, context: &Context, command: &Command) -> String {
//         unimplemented!()
//     }
//
//     fn kind(&self) -> HelpKind {
//         unimplemented!()
//     }
// }