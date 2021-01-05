#![warn(clippy::manual_unwrap_or)]
use std::fmt::{Display, Formatter};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive, Sub, RangeBounds};
use std::collections::Bound;

/**
Represents the number of arguments a option or command can take.
Numeric signed, unsigned and ranges of these types implements `Into<ArgCount>`.
*/
#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ArgCount {
    min: Option<usize>,
    max: Option<usize>,
}

impl ArgCount {
    /// Constructs a new `ArgCount` with the specified `min` and `max` argument count.
    #[inline]
    pub fn new(min: usize, max: usize) -> Self {
        Self::new_checked(Some(min), Some(max)).expect("min < max")
    }

    /// Constructs a new `ArgCount` or returns `None` if min > max
    #[inline]
    pub fn new_checked(min: Option<usize>, max: Option<usize>) -> Option<Self> {
        match (min, max) {
            (Some(min), Some(max)) if min > max => None,
            _ => unsafe { Some(Self::new_unchecked(min, max)) }
        }
    }

    #[inline(always)]
    const unsafe fn new_unchecked(min: Option<usize>, max: Option<usize>) -> Self {
        ArgCount { min, max }
    }

    /// Constructs a new `ArgCount` for not arguments.
    #[inline]
    pub const fn zero() -> Self {
        unsafe { Self::new_unchecked(Some(0), Some(0)) }
    }

    /// Constructs a new `ArgCount` for exactly 1 argument.
    #[inline]
    pub const fn one() -> Self {
        unsafe { Self::new_unchecked(Some(1), Some(1)) }
    }

    /// Constructs a new `ArgCount` for any number of arguments.
    #[inline]
    pub const fn any() -> Self {
        unsafe { Self::new_unchecked(None, None) }
    }

    /// Constructs a new `ArgCount` for the specified number of arguments.
    #[inline]
    pub const fn exactly(count: usize) -> Self {
        unsafe { Self::new_unchecked(Some(count), Some(count)) }
    }

    /// Constructs a new `ArgCount` for more than the specified number of arguments.
    #[inline]
    pub fn more_than(min: usize) -> Self {
        unsafe { Self::new_unchecked(Some(min), None) }
    }

    /// Constructs a new `ArgCount` for less than the specified number of arguments.
    #[inline]
    pub fn less_than(max: usize) -> Self {
        unsafe { Self::new_unchecked(None, Some(max)) }
    }

    /// Returns the min number of arguments can takes.
    #[inline]
    pub const fn min(&self) -> usize {
        match self.min {
            Some(n) => n,
            None => 0
        }
    }

    /// Returns the max number of arguments can takes.
    #[inline]
    pub const fn max(&self) -> usize {
        match self.max {
            Some(n) => n,
            None => usize::max_value()
        }
    }

    /// Returns the `min` number of arguments or `None` if unbounded.
    #[inline]
    pub const fn min_count(&self) -> Option<usize> {
        self.min
    }

    /// Returns the `max` number of arguments of `None` if unbounded.
    #[inline]
    pub const fn max_count(&self) -> Option<usize> {
        self.max
    }

    /// Returns `true` if this takes the provided number of arguments.
    #[inline]
    pub const fn takes(&self, count: usize) -> bool {
        count >= self.min() && count <= self.max()
    }

    /// Returns `true` if this takes arguments.
    #[inline]
    pub const fn takes_args(&self) -> bool {
        self.max() != 0
    }

    /// Returns `true` if this takes no arguments.
    #[inline]
    pub const fn takes_no_args(&self) -> bool {
        self.min() == 0 && self.max() == 0
    }

    /// Returns `true` if this takes an exact number of arguments.
    #[inline]
    pub const fn is_exact(&self) -> bool {
        self.min() == self.max()
    }

    /// Returns `true` if this takes exactly the specified number of arguments.
    #[inline]
    pub const fn takes_exactly(&self, count: usize) -> bool {
        self.min() == count && self.max() == count
    }
}

impl Display for ArgCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_exact() {
            return if self.takes_exactly(0){
                write!(f, "no arguments")
            } else if self.takes_exactly(1){
                write!(f, "1 argument")
            } else {
                write!(f, "{} arguments", self.min())
            };
        }

        match (self.min, self.max) {
            (Some(min), Some(max)) => write!(f, "{} to {} arguments", min, max),
            (Some(min), None) => write!(f, "{} or more arguments", min),
            (None, Some(max)) => write!(f, "{} or less arguments", max),
            (None, None) => write!(f, "any number of arguments")
        }
    }
}

impl From<ArgCount> for RangeInclusive<usize> {
    fn from(arg_count: ArgCount) -> Self {
        arg_count.min()..=arg_count.max()
    }
}

impl From<RangeFull> for ArgCount {
    fn from(_: RangeFull) -> Self {
        ArgCount::any()
    }
}

impl RangeBounds<usize> for ArgCount {
    fn start_bound(&self) -> Bound<&usize> {
        match self.min {
            Some(ref n) => Bound::Included(n),
            None => Bound::Unbounded
        }
    }

    fn end_bound(&self) -> Bound<&usize> {
        match self.max {
            Some(ref n) => Bound::Included(n),
            None => Bound::Unbounded
        }
    }
}

macro_rules! impl_arg_count_from_unsigned_int {
    ($($target:ident),*) => {
        $(
            impl From<$target> for ArgCount {
                fn from(value: $target) -> ArgCount {
                    ArgCount::exactly(value as usize)
                }
            }

            impl From<RangeInclusive<$target>> for ArgCount {
                fn from(value: RangeInclusive<$target>) -> ArgCount {
                    let start = *value.start();
                    let end = *value.end();
                    ArgCount::new(start as usize, end as usize)
                }
            }

            impl From<Range<$target>> for ArgCount {
                fn from(value: Range<$target>) -> ArgCount {
                    let start = value.start;
                    let end = value.end.sub(1);
                    ArgCount::new(start as usize, end as usize)
                }
            }

            impl From<RangeFrom<$target>> for ArgCount {
                fn from(value: RangeFrom<$target>) -> ArgCount {
                    let start = value.start;
                    ArgCount::more_than(start as usize)
                }
            }

            impl From<RangeTo<$target>> for ArgCount {
                fn from(value: RangeTo<$target>) -> ArgCount {
                    let end = value.end.sub(1);
                    ArgCount::less_than(end as usize)
                }
            }

            impl From<RangeToInclusive<$target>> for ArgCount {
                fn from(value: RangeToInclusive<$target>) -> ArgCount {
                    let end = value.end;
                    ArgCount::less_than(end as usize)
                }
            }
        )*
    };
}

macro_rules! impl_arg_count_from_signed_int {
    ($($target:ident),*) => {
        $(
            impl From<$target> for ArgCount {
                fn from(value: $target) -> ArgCount {
                    assert!(value >= 0, "argument count cannot be negative: {}", value);
                    ArgCount::exactly(value as usize)
                }
            }

            impl From<RangeInclusive<$target>> for ArgCount {
                fn from(value: RangeInclusive<$target>) -> ArgCount {
                    let start = *value.start();
                    let end = *value.end();

                    assert!(start >= 0, "start cannot be negative");
                    assert!(end >= 0, "end cannot be negative");

                    ArgCount::new(start as usize, end as usize)
                }
            }

            impl From<Range<$target>> for ArgCount {
                fn from(value: Range<$target>) -> ArgCount {
                    let start = value.start;
                    let end = value.end.sub(1);

                    assert!(start >= 0, "start cannot be negative");
                    assert!(end >= 0, "end cannot be negative");

                    ArgCount::new(start as usize, end as usize)
                }
            }

            impl From<RangeFrom<$target>> for ArgCount {
                fn from(value: RangeFrom<$target>) -> ArgCount {
                    let start = value.start;
                    assert!(start >= 0, "start cannot be negative");
                    ArgCount::more_than(start as usize)
                }
            }

            impl From<RangeTo<$target>> for ArgCount {
                fn from(value: RangeTo<$target>) -> ArgCount {
                    let end = value.end.sub(1);
                    assert!(end >= 0, "end cannot be negative");
                    ArgCount::less_than(end as usize)
                }
            }

            impl From<RangeToInclusive<$target>> for ArgCount {
                fn from(value: RangeToInclusive<$target>) -> ArgCount {
                    let end = value.end;
                    assert!(end >= 0, "end cannot be negative");
                    ArgCount::less_than(end as usize)
                }
            }
        )*
    };
}

impl_arg_count_from_unsigned_int! { u8, u16, u32, u64, u128, usize }

impl_arg_count_from_signed_int! { i8, i16, i32, i64, i128, isize }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arg_count_test() {
        let arg_count = ArgCount::new(2, 5);
        assert_eq!(arg_count.min(), 2);
        assert_eq!(arg_count.max(), 5);
        assert!(arg_count.takes_args());
        assert!(arg_count.takes(2));
        assert!(arg_count.takes(3));
        assert!(arg_count.takes(4));
        assert!(arg_count.takes(5));
    }

    #[test]
    fn into_arg_count_test() {
        fn assert_into<A: Into<ArgCount>>(value: A, expected_min: usize, expected_max: usize) {
            let arg_count = value.into();
            let type_name = std::any::type_name::<A>();
            assert_eq!(
                arg_count.min(),
                expected_min,
                "min value - type: `{}`",
                type_name
            );
            assert_eq!(
                arg_count.max(),
                expected_max,
                "max value - type: `{}`",
                type_name
            );
        }

        assert_into(2, 2, 2);
        assert_into(.., 0, usize::max_value());
        assert_into(10.., 10, usize::max_value());
        assert_into(..20, 0, 19);
        assert_into(..=20, 0, 20);
        assert_into(1..10, 1, 9);
        assert_into(1..=10, 1, 10);

        assert_into(0..1, 0, 0);
    }

    #[test]
    #[should_panic]
    fn into_arg_count_panic_test1() {
        let _: ArgCount = (-1_i32).into();
    }

    #[test]
    #[should_panic]
    fn into_arg_count_panic_test2() {
        let _: ArgCount = (1..1).into();
    }

    #[test]
    #[should_panic]
    fn into_arg_count_panic_test3() {
        let _: ArgCount = (0..-2).into();
    }

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
        assert!(arg_count.takes(0));
        assert!(arg_count.takes(1));
        assert!(arg_count.takes(2));
        assert!(arg_count.takes(3));
    }

    #[test]
    fn display_test() {
        assert_eq!(ArgCount::new(0, 2).to_string(), "0 to 2 arguments");
        assert_eq!(ArgCount::exactly(1).to_string(), "1 argument");
        assert_eq!(ArgCount::exactly(3).to_string(), "3 arguments");
    }
}
