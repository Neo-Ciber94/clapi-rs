#![allow(dead_code)]
#![allow(unused_must_use)]

mod inner_cell;
mod lazy;
mod once_cell;

/// Extension methods.
mod ext;
pub use ext::*;

#[macro_use]
mod macros;
pub use macros::*;

/// Performs an operation over a value of type `T` and returns the result.
///
/// This trait is implemented for all types.
pub trait Then: Sized {
    #[inline]
    fn then<'a, R, F: Fn(&'a Self) -> R>(&'a self, f: F) -> R {
        f(self)
    }

    #[inline]
    fn then_mut<'a, R, F: FnMut(&'a mut Self) -> R>(&'a mut self, mut f: F) -> R {
        f(self)
    }

    #[inline]
    fn then_apply<R, F: FnOnce(Self) -> R>(self, f: F) -> R {
        f(self)
    }
}

/// Performs an operation over a value of type `T` and returns `Self`.
///
/// This trait is implemented for all types.
pub trait Also: Sized {
    #[inline]
    fn also<R, F: Fn(&Self) -> R>(self, f: F) -> Self {
        f(&self);
        self
    }

    #[inline]
    fn also_mut<R, F: Fn(&mut Self) -> R>(mut self, f: F) -> Self {
        f(&mut self);
        self
    }
}

impl<T> Then for T {}
impl<T> Also for T {}

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
