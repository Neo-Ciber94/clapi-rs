use std::fmt::{Display, Formatter};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/**
Represents the number of arguments a option or command can take.
Numeric signed, unsigned and ranges of these types implements `Into<ArgCount>`.
*/
#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ArgCount {
    min_arg_count: usize,
    max_arg_count: usize,
}

impl ArgCount {
    /// Constructs a new `ArgCount` with the specified `min` and `max` argument count.
    pub fn new(min_arg_count: usize, max_arg_count: usize) -> Self {
        assert!(
            min_arg_count <= max_arg_count,
            "min cannot be greater than max argument count"
        );
        ArgCount {
            min_arg_count,
            max_arg_count,
        }
    }

    #[inline]
    const fn new_unchecked(min_arg_count: usize, max_arg_count: usize) -> Self {
        ArgCount {
            min_arg_count,
            max_arg_count,
        }
    }

    /// Constructs a new `ArgCount` for not arguments.
    #[inline]
    pub const fn zero() -> Self {
        Self::new_unchecked(0, 0)
    }

    /// Constructs a new `ArgCount` for exactly 1 argument.
    #[inline]
    pub const fn one() -> Self {
        Self::new_unchecked(1, 1)
    }

    /// Constructs a new `ArgCount` for any number of arguments.
    #[inline]
    pub const fn any() -> Self {
        Self::new_unchecked(0, usize::max_value())
    }

    /// Constructs a new `ArgCount` for the specified number of arguments.
    #[inline]
    pub const fn exactly(arg_count: usize) -> Self {
        Self::new_unchecked(arg_count, arg_count)
    }

    /// Constructs a new `ArgCount` for more than the specified number of arguments.
    #[inline]
    pub const fn more_than(min_arg_count: usize) -> Self {
        Self::new_unchecked(min_arg_count, usize::max_value())
    }

    /// Constructs a new `ArgCount` for less than the specified number of arguments.
    #[inline]
    pub const fn less_than(max_arg_count: usize) -> Self {
        Self::new_unchecked(0, max_arg_count)
    }

    /// Returns the min number of arguments can takes.
    #[inline]
    pub const fn min_arg_count(&self) -> usize {
        self.min_arg_count
    }

    /// Returns the max number of arguments can takes.
    #[inline]
    pub const fn max_arg_count(&self) -> usize {
        self.max_arg_count
    }

    /// Returns `true` if this accept the provided number of arguments.
    #[inline]
    pub fn contains(&self, value: usize) -> bool {
        (self.min_arg_count..=self.max_arg_count).contains(&value)
    }

    /// Returns `true` if this takes arguments.
    #[inline]
    pub const fn takes_args(&self) -> bool {
        self.max_arg_count != 0
    }

    /// Returns `true` if this takes an exact number of arguments.
    #[inline]
    pub const fn is_exact(&self) -> bool {
        self.min_arg_count == self.max_arg_count
    }

    /// Returns `true` if this takes exactly the specified number of arguments.
    #[inline]
    pub fn takes_exactly(&self, arg_count: usize) -> bool {
        self.min_arg_count == arg_count && self.max_arg_count == arg_count
    }
}

impl Display for ArgCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_exact() {
            write!(
                f,
                "{} argument{}",
                self.min_arg_count,
                if self.takes_exactly(1) { "" } else { "s" }
            )
        } else {
            write!(
                f,
                "{} to {} arguments",
                self.min_arg_count, self.max_arg_count
            )
        }
    }
}

impl Into<RangeInclusive<usize>> for ArgCount {
    fn into(self) -> RangeInclusive<usize> {
        self.min_arg_count..=self.max_arg_count
    }
}

impl Into<ArgCount> for RangeFull {
    #[inline]
    fn into(self) -> ArgCount {
        ArgCount::any()
    }
}

macro_rules! impl_into_for_signed {
    ($($target:ident),*) => {
        $(
            impl Into<ArgCount> for $target{
                fn into(self) -> ArgCount {
                    assert!(self >= 0, "argument count cannot be negative: {}", self);
                    ArgCount{
                        min_arg_count: self as usize,
                        max_arg_count: self as usize
                    }
                }
            }
        )*
    };
}

macro_rules! impl_into_for_unsigned {
    ($($target:ident),*) => {
        $(
            impl Into<ArgCount> for $target{
                fn into(self) -> ArgCount {
                    ArgCount{
                        min_arg_count: self as usize,
                        max_arg_count: self as usize
                    }
                }
            }
        )*
    };
}

impl_into_for_signed! {i8, i16, i32, i64, i128, isize}

impl_into_for_unsigned! {u8, u16, u32, u64, u128, usize}

macro_rules! impl_into_for_unsigned_range {
    ($($target:ident),*) => {
        $(
            impl Into<ArgCount> for RangeInclusive<$target>{
                fn into(self) -> ArgCount {
                    ArgCount{
                        min_arg_count: *self.start() as usize,
                        max_arg_count: *self.end() as usize
                    }
                }
            }

            impl Into<ArgCount> for Range<$target>{
                fn into(self) -> ArgCount {
                    ArgCount{
                        min_arg_count: self.start as usize,
                        max_arg_count: self.end as usize
                    }
                }
            }

            impl Into<ArgCount> for RangeFrom<$target>{
                fn into(self) -> ArgCount {
                    ArgCount::more_than(self.start as usize)
                }
            }

            impl Into<ArgCount> for RangeTo<$target>{
                fn into(self) -> ArgCount {
                    ArgCount::less_than(self.end as usize)
                }
            }

            impl Into<ArgCount> for RangeToInclusive<$target>{
                fn into(self) -> ArgCount {
                    ArgCount::less_than(self.end as usize)
                }
            }
        )*
    };
}

macro_rules! impl_into_for_signed_range {
    ($($target:ident),*) => {
        $(
            impl Into<ArgCount> for RangeInclusive<$target>{
                fn into(self) -> ArgCount {
                    let start = *self.start();
                    let end = *self.end();

                    assert!(start >= 0, "start cannot be negative");
                    assert!(end >= 0, "end cannot be negative");

                    ArgCount{
                        min_arg_count: start as usize,
                        max_arg_count: end as usize
                    }
                }
            }

            impl Into<ArgCount> for Range<$target>{
                fn into(self) -> ArgCount {
                    let start = self.start;
                    let end = self.end;

                    assert!(start >= 0, "start cannot be negative");
                    assert!(end >= 0, "end cannot be negative");

                    ArgCount{
                        min_arg_count: start as usize,
                        max_arg_count: end as usize
                    }
                }
            }

            impl Into<ArgCount> for RangeFrom<$target>{
                fn into(self) -> ArgCount {
                    let start = self.start;
                    assert!(start >= 0, "start cannot be negative");

                    ArgCount::more_than(start as usize)
                }
            }

            impl Into<ArgCount> for RangeTo<$target>{
                fn into(self) -> ArgCount {
                    let end = self.end;
                    assert!(end >= 0, "end cannot be negative");

                    ArgCount::less_than(end as usize)
                }
            }

            impl Into<ArgCount> for RangeToInclusive<$target>{
                fn into(self) -> ArgCount {
                    let end = self.end;
                    assert!(end >= 0, "end cannot be negative");

                    ArgCount::less_than(end as usize)
                }
            }
        )*
    };
}

impl_into_for_unsigned_range! { u8, u16, u32, u64, u128, usize }

impl_into_for_signed_range! { i8, i16, i32, i64, i128, isize }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_test() {
        let arg_count = ArgCount::zero();
        assert!(!arg_count.takes_args());
        assert!(arg_count.is_exact());
        assert_eq!(arg_count.min_arg_count, 0);
        assert_eq!(arg_count.max_arg_count, 0);
    }

    #[test]
    fn one_test() {
        let arg_count = ArgCount::one();
        assert!(arg_count.takes_args());
        assert!(arg_count.is_exact());
        assert_eq!(arg_count.min_arg_count, 1);
        assert_eq!(arg_count.max_arg_count, 1);
    }

    #[test]
    fn any_test() {
        let arg_count = ArgCount::any();
        assert!(arg_count.takes_args());
        assert!(!arg_count.is_exact());
        assert_eq!(arg_count.min_arg_count, 0);
        assert_eq!(arg_count.max_arg_count, usize::max_value());
    }

    #[test]
    fn exactly_test() {
        let arg_count = ArgCount::exactly(2);
        assert!(arg_count.takes_exactly(2));
        assert_eq!(arg_count.min_arg_count, 2);
        assert_eq!(arg_count.max_arg_count, 2);
    }

    #[test]
    fn more_than_test() {
        let arg_count = ArgCount::more_than(1);
        assert!(!arg_count.takes_exactly(1));
        assert_eq!(arg_count.min_arg_count, 1);
        assert_eq!(arg_count.max_arg_count, usize::max_value());
    }

    #[test]
    fn less_than_test() {
        let arg_count = ArgCount::less_than(5);
        assert!(!arg_count.takes_exactly(5));
        assert_eq!(arg_count.min_arg_count, 0);
        assert_eq!(arg_count.max_arg_count, 5);
    }

    #[test]
    fn contains_test() {
        let arg_count = ArgCount::new(0, 3);
        assert!(arg_count.contains(0));
        assert!(arg_count.contains(1));
        assert!(arg_count.contains(2));
        assert!(arg_count.contains(3));
    }

    #[test]
    fn display_test() {
        assert_eq!(ArgCount::new(0, 2).to_string(), "0 to 2 arguments");
        assert_eq!(ArgCount::exactly(1).to_string(), "1 argument");
        assert_eq!(ArgCount::exactly(3).to_string(), "3 arguments");
    }
}
