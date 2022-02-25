mod month_ext;
mod print;
mod string_ext;
mod weekday_ext;

use crate::month_ext::MonthExt;
use crate::string_ext::StringExt;
use crate::weekday_ext::WeekdayExt;
use chrono::{DateTime, Datelike, Local, Month, Timelike};
use clapi::macros::*;

#[command(name = "getdate", description = "Prints the current date and time")]
#[option(
    format,
    alias = "f",
    description = "The format to print the date and time in"
)]
#[option(
    no_color,
    name = "no-color",
    description = "Disables colored output",
    global = true
)]
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
fn day(no_color: bool) {
    println_colored!(no_color, "{}", now().day());
}

#[subcommand(description = "Prints the current month")]
fn month(no_color: bool) {
    println_colored!(no_color, "{}", now().month());
}

#[subcommand(description = "Prints the current year")]
fn year(no_color: bool) {
    println_colored!(no_color, "{}", now().year());
}

fn now() -> DateTime<Local> {
    Local::now()
}

fn format_date(date: &DateTime<Local>, format: String) -> String {
    let mut result = format.clone();

    // Set year
    result = result.replace("%YYYY", &date.year().to_string());
    result = result.replace("%YY", &date.year().to_string()[2..]);

    // Set month
    result = result.replace("%MMMM", Month::from_month(date.month()).unwrap().name());
    result = result.replace(
        "%MMM",
        Month::from_month(date.month()).unwrap().name_short(),
    );
    result = result.replace("%MM", &date.month().to_string().pad_left(2, '0'));
    result = result.replace("%M", &date.month().to_string());

    // Set day
    result = result.replace("%DDDD", &date.weekday().name_long());
    result = result.replace("%DDD", &date.weekday().to_string());
    result = result.replace("%DD", &date.day().to_string().pad_left(2, '0'));
    result = result.replace("%D", &date.day().to_string());

    // Set hours
    result = result.replace("%hh", &date.hour().to_string().pad_left(2, '0'));
    result = result.replace("%h", &date.hour().to_string());

    // Set minutes
    result = result.replace("%mm", &date.minute().to_string().pad_left(2, '0'));
    result = result.replace("%m", &date.minute().to_string());

    // Set seconds
    result = result.replace("%ss", &date.second().to_string().pad_left(2, '0'));
    result = result.replace("%s", &date.second().to_string());

    // Set milliseconds
    result = result.replace(
        "%zz",
        &*nanos_to_millis(date.nanosecond())
            .to_string()
            .pad_left(2, '0'),
    );
    result = result.replace("%z", &*nanos_to_millis(date.nanosecond()).to_string());

    result
}

fn nanos_to_millis(nanos: u32) -> u32 {
    nanos / 1_000_000
}
