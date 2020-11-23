use std::fmt::{Display, Formatter};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/**
Represents the number of arguments a option or command can take.
Numeric signed, unsigned and ranges of these types implements `Into<ArgCount>`.
*/
#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ArgCount {
    min: usize,
    max: usize,
}

impl ArgCount {
    /// Constructs a new `ArgCount` with the specified `min` and `max` argument count.
    pub fn new(min: usize, max: usize) -> Self {
        assert!(min <= max);
        Self::new_unchecked(min, max)
    }

    #[inline(always)]
    const fn new_unchecked(min: usize, max: usize) -> Self {
        ArgCount { min, max, }
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
    pub const fn exactly(count: usize) -> Self {
        Self::new_unchecked(count, count)
    }

    /// Constructs a new `ArgCount` for more than the specified number of arguments.
    #[inline]
    pub const fn more_than(min: usize) -> Self {
        Self::new_unchecked(min, usize::max_value())
    }

    /// Constructs a new `ArgCount` for less than the specified number of arguments.
    #[inline]
    pub const fn less_than(max: usize) -> Self {
        Self::new_unchecked(0, max)
    }

    /// Returns the min number of arguments can takes.
    #[inline]
    pub const fn min(&self) -> usize {
        self.min
    }

    /// Returns the max number of arguments can takes.
    #[inline]
    pub const fn max(&self) -> usize {
        self.max
    }

    /// Returns `true` if this accept the provided number of arguments.
    #[inline]
    pub fn contains(&self, value: usize) -> bool {
        (self.min..=self.max).contains(&value)
    }

    /// Returns `true` if this takes arguments.
    #[inline]
    pub const fn takes_args(&self) -> bool {
        self.max != 0
    }

    /// Returns `true` if this takes an exact number of arguments.
    #[inline]
    pub const fn is_exact(&self) -> bool {
        self.min == self.max
    }

    /// Returns `true` if this takes exactly the specified number of arguments.
    #[inline]
    pub fn takes_exactly(&self, arg_count: usize) -> bool {
        self.min == arg_count && self.max == arg_count
    }
}

impl Display for ArgCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_exact() {
            write!(
                f,
                "{} argument{}",
                self.min,
                if self.takes_exactly(1) { "" } else { "s" }
            )
        } else {
            write!(
                f,
                "{} to {} arguments",
                self.min, self.max
            )
        }
    }
}

impl Into<RangeInclusive<usize>> for ArgCount {
    fn into(self) -> RangeInclusive<usize> {
        self.min..=self.max
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
                        min: self as usize,
                        max: self as usize
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
                        min: self as usize,
                        max: self as usize
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
                        min: *self.start() as usize,
                        max: *self.end() as usize
                    }
                }
            }

            impl Into<ArgCount> for Range<$target>{
                fn into(self) -> ArgCount {
                    ArgCount{
                        min: self.start as usize,
                        max: self.end as usize
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
                        min: start as usize,
                        max: end as usize
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
                        min: start as usize,
                        max: end as usize
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
        assert_eq!(arg_count.min(), 0);
        assert_eq!(arg_count.max(), 0);
    }

    #[test]
    fn one_test() {
        let arg_count = ArgCount::one();
        assert!(arg_count.takes_args());
        assert!(arg_count.is_exact());
        assert_eq!(arg_count.min(), 1);
        assert_eq!(arg_count.max(), 1);
    }

    #[test]
    fn any_test() {
        let arg_count = ArgCount::any();
        assert!(arg_count.takes_args());
        assert!(!arg_count.is_exact());
        assert_eq!(arg_count.min(), 0);
        assert_eq!(arg_count.max(), usize::max_value());
    }

    #[test]
    fn exactly_test() {
        let arg_count = ArgCount::exactly(2);
        assert!(arg_count.takes_exactly(2));
        assert_eq!(arg_count.min(), 2);
        assert_eq!(arg_count.max(), 2);
    }

    #[test]
    fn more_than_test() {
        let arg_count = ArgCount::more_than(1);
        assert!(!arg_count.takes_exactly(1));
        assert_eq!(arg_count.min(), 1);
        assert_eq!(arg_count.max(), usize::max_value());
    }

    #[test]
    fn less_than_test() {
        let arg_count = ArgCount::less_than(5);
        assert!(!arg_count.takes_exactly(5));
        assert_eq!(arg_count.min(), 0);
        assert_eq!(arg_count.max(), 5);
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
