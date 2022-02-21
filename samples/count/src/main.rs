use clapi::macros::*;

#[command]
#[option(val, global = true, description = "The value to display")]
fn main(val: Option<i64>) {
    println!("{:?}", val);
}

#[subcommand]
#[option(other, from_global = true)]
fn twice(other: Option<i64>) {
    println!("{:?} - {:?}", other, other);
}

//#[command]
// fn __main() {
//     CommandLine::new(
//         Command::new("counter")
//             .option(
//                 CommandOption::new("separator").global(true).alias("s").arg(
//                     Argument::with_name("separator")
//                         .description("The separator to use between the numbers"),
//                 ),
//             )
//             .handler(|opts, args| {
//                 let separator = opts
//                     .get_arg("separator")
//                     .map(|s| s.convert::<String>().ok())
//                     .flatten()
//                     .unwrap_or("\n".to_string());
//                 let range = args.get_raw_args_as_type::<i64>()?;
//
//                 assert!(range.len() > 0, "Expected 1 or more numbers");
//
//                 let min: i64;
//                 let max: i64;
//
//                 if range.len() == 1 {
//                     min = 0;
//                     max = range[0];
//                 } else {
//                     min = range[0];
//                     max = range[1];
//                 }
//
//                 for i in min..=max {
//                     print!("{}", i);
//
//                     if i != max {
//                         print!("{}", separator);
//                     }
//                 }
//
//                 Ok(())
//             })
//             .arg(
//                 Argument::with_name("range")
//                     .description("Range of numbers")
//                     .min_values(1)
//                     .max_values(2)
//                     .validator(validate_type::<u64>()),
//             )
//             .subcommand(
//                 Command::new("reverse")
//                     .arg(
//                         Argument::with_name("range")
//                             .description("Range of numbers")
//                             .min_values(1)
//                             .max_values(2)
//                             .validator(validate_type::<u64>()),
//                     )
//                     .handler(|opts, args| {
//                         let separator = opts
//                             .get_arg("separator")
//                             .map(|s| s.convert::<String>().ok())
//                             .flatten()
//                             .unwrap_or("\n".to_string());
//                         let range = args.get_raw_args_as_type::<i64>()?;
//
//                         assert!(range.len() > 0, "Expected 1 or more numbers");
//
//                         let min: i64;
//                         let mut max: i64;
//
//                         if range.len() == 1 {
//                             min = 0;
//                             max = range[0];
//                         } else {
//                             min = range[0];
//                             max = range[1];
//                         }
//
//                         while min <= max {
//                             print!("{}", max);
//
//                             if min < max {
//                                 print!("{}", separator);
//                             }
//
//                             max -= 1;
//                         }
//
//                         Ok(())
//                     }),
//             ),
//     )
//     .use_default_help()
//     .use_default_suggestions()
//     .run()
//     .map_err(|e| e.exit())
//     .unwrap()
// }
