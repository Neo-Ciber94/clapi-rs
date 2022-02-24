use chrono::Weekday;

pub trait WeekdayExt {
    fn name_long(&self) -> &'static str;
}

impl WeekdayExt for Weekday {
    fn name_long(&self) -> &'static str {
        match { self.num_days_from_monday() } {
            0 => "Monday",
            1 => "Tuesday",
            2 => "Wednesday",
            3 => "Thursday",
            4 => "Friday",
            5 => "Saturday",
            6 => "Sunday",
            _ => unreachable!(),
        }
    }
}
