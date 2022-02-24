mod month_ext;
mod print;
mod string_ext;
mod weekday_ext;

use crate::month_ext::MonthExt;
use crate::string_ext::StringExt;
use crate::weekday_ext::WeekdayExt;
use chrono::{DateTime, Datelike, Local, Month};
use clapi::macros::*;

#[command(name = "datetime", description = "Prints the current date and time")]
#[option(
    format,
    alias = "f",
    description = "The format to print the date and time in"
)]
#[option(no_color, name = "no-color", description = "Disables colored output")]
pub fn main(format: Option<String>, no_color: bool) {
    let now = now();
    if let Some(format) = format {
        println_colored!(!no_color, "{}", format_date(&now, format));
    } else {
        println_colored!(!no_color, "{}", &now.to_rfc2822());
    }
}

#[subcommand(description = "Prints the current weekday")]
fn weekday(no_color: bool) {
    println_colored!(!no_color, "{}", now().weekday().name_long());
}

#[subcommand(description = "Prints the current day")]
fn day() {
    println!("{}", now().day());
}

#[subcommand(description = "Prints the current month")]
fn month() {
    println!("{}", now().month());
}

#[subcommand(description = "Prints the current year")]
fn year() {
    println!("{}", now().year());
}

fn now() -> DateTime<Local> {
    Local::now()
}

fn format_date(date: &DateTime<Local>, mut format: String) -> String {
    let mut result = format.clone();

    // Set year
    result = result.replace("YYYY", &date.year().to_string());
    result = result.replace("YY", &date.year().to_string()[2..]);

    // Set month
    result = result.replace("MMMM", Month::from_month(date.month()).unwrap().name());
    result = result.replace("MMM", Month::from_month(date.month()).unwrap().name_short());
    result = result.replace("MM", &date.month().to_string().pad_left(2, '0'));
    result = result.replace("M", &date.month().to_string());

    // Set day
    result = result.replace("DDDD", &date.weekday().name_long());
    result = result.replace("DDD", &date.weekday().to_string());
    result = result.replace("DD", &date.day().to_string().pad_left(2, '0'));
    result = result.replace("D", &date.day().to_string());

    // Set hours

    // Set minutes

    // Set seconds

    // Set milliseconds

    result
}
