#![allow(dead_code)]
#![allow(unused_must_use)]

pub use debug_utils::*;
mod debug_utils {
    use std::fmt::{Debug, Formatter};

    pub fn debug_option<'a, T>(option: &'a Option<T>, if_some: &'a str) -> impl Debug + 'a {
        struct OptionDebug<'a, T> {
            option: &'a Option<T>,
            if_some: &'a str,
        }

        impl<T> Debug for OptionDebug<'_, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match &self.option {
                    None => write!(f, "None"),
                    Some(_) => write!(f, "Some({})", self.if_some),
                }
            }
        }

        OptionDebug { option, if_some }
    }
}
