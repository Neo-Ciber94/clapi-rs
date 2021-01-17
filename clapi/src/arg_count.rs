#![allow(clippy::manual_unwrap_or)]
use std::fmt::{Display, Formatter};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive, Sub, RangeBounds};
use std::collections::Bound;

/*
#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq)]
struct NumValues {
    min_values: Option<usize>,
    max_values: Option<usize>,
}
*/

/**
Represents the number of values an argument takes.
*/
#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ArgCount {
    min: Option<usize>,
    max: Option<usize>,
}

impl ArgCount {
    /// Constructs a new `ArgCount` with the given min and max.
    /// It takes `Option<usize>` where `Some(usize)` is bounded and `None` is unbounded.
    ///
    /// # Example
    /// ```
    /// use clapi::ArgCount;
    ///
    /// // This goes from 2 to the usize::MAX
    /// let unbounded_max = ArgCount::new(Some(2), None);
    /// assert_eq!(unbounded_max.min_or_default(), 2);
    /// assert_eq!(unbounded_max.max_or_default(), usize::MAX);
    ///
    /// // This goes from usize::MIN to 12
    /// let unbounded_min = ArgCount::new(None, Some(12));
    /// assert_eq!(unbounded_min.min_or_default(), usize::MIN);
    /// assert_eq!(unbounded_min.max_or_default(), 12);
    ///
    /// // This goes from 5 to 10
    /// let bounded = ArgCount::new(Some(5), Some(10));
    /// assert_eq!(bounded.min_or_default(), 5);
    /// assert_eq!(bounded.max_or_default(), 10);
    /// ```
    ///
    /// # Panics
    /// If min > max.
    #[inline]
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        if let (Some(min), Some(max)) = (min, max) {
            assert!(min <= max, "min cannot be greater than max");
            unsafe { Self::new_unchecked(Some(min), Some(max)) }
        } else {
            unsafe { Self::new_unchecked(min, max) }
        }
    }

    /// Constructs a new `ArgCount` with a know `min` and `max`
    ///
    /// # Example
    /// ```
    /// use clapi::ArgCount;
    ///
    /// let count = ArgCount::new_bounded(2, 10);
    /// assert_eq!(count.min_or_default(), 2);
    /// assert_eq!(count.max_or_default(), 10);
    /// ```
    ///
    /// # Panics
    /// If min > max
    #[inline]
    pub fn new_bounded(min: usize, max: usize) -> Self {
        Self::new(Some(min), Some(max))
    }

    #[inline(always)]
    const unsafe fn new_unchecked(min: Option<usize>, max: Option<usize>) -> Self {
        ArgCount { min, max }
    }

    /// Constructs a new `ArgCount` for not values.
    #[inline]
    pub const fn zero() -> Self {
        unsafe { Self::new_unchecked(Some(0), Some(0)) }
    }

    /// Constructs a new `ArgCount` for exactly 1 values.
    #[inline]
    pub const fn one() -> Self {
        unsafe { Self::new_unchecked(Some(1), Some(1)) }
    }

    /// Constructs a new `ArgCount` for any number of values.
    #[inline]
    pub const fn any() -> Self {
        unsafe { Self::new_unchecked(None, None) }
    }

    /// Constructs a new `ArgCount` for the specified number of values.
    #[inline]
    pub const fn exactly(count: usize) -> Self {
        unsafe { Self::new_unchecked(Some(count), Some(count)) }
    }

    /// Constructs a new `ArgCount` for more than the specified number of values.
    #[inline]
    pub fn more_than(min: usize) -> Self {
        unsafe { Self::new_unchecked(Some(min), None) }
    }

    /// Constructs a new `ArgCount` for less than the specified number of values.
    #[inline]
    pub fn less_than(max: usize) -> Self {
        unsafe { Self::new_unchecked(None, Some(max)) }
    }

    /// Returns the min number of values if bounded or `usize::MIN` if unbounded.
    #[inline]
    pub const fn min_or_default(&self) -> usize {
        match self.min {
            Some(n) => n,
            None => usize::MIN
        }
    }

    /// Returns the max number of values if bounded or `usize::MAX` if unbounded.
    #[inline]
    pub const fn max_or_default(&self) -> usize {
        match self.max {
            Some(n) => n,
            None => usize::MAX
        }
    }

    /// Returns the `min` number of values or `None` if unbounded.
    #[inline]
    pub const fn min(&self) -> Option<usize> {
        self.min
    }

    /// Returns the `max` number of values of `None` if unbounded.
    #[inline]
    #[inline]
    pub const fn max(&self) -> Option<usize> {
        self.max
    }

    /// Returns a copy of this `ArgCount` with the given `min`.
    #[inline]
    pub fn with_min(&self, min: usize) -> Self {
        Self::new(Some(min), self.max)
    }

    /// Returns a copy of this `ArgCount` with the given `max`.
    #[inline]
    pub fn with_max(&self, max: usize) -> Self {
        Self::new(self.min, Some(max))
    }

    /// Returns `true` if this takes the provided number of values.
    #[inline]
    pub const fn takes(&self, count: usize) -> bool {
        count >= self.min_or_default() && count <= self.max_or_default()
    }

    /// Returns `true` if this takes values.
    #[inline]
    pub const fn takes_values(&self) -> bool {
        self.max_or_default() != 0
    }

    /// Returns `true` if this takes an exact number of values.
    #[inline]
    pub const fn is_exact(&self) -> bool {
        self.min_or_default() == self.max_or_default()
    }

    /// Returns `true` if this takes exactly the specified number of values.
    #[inline]
    pub const fn takes_exactly(&self, count: usize) -> bool {
        self.min_or_default() == count && self.max_or_default() == count
    }
}

impl Display for ArgCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_exact() {
            return if self.takes_exactly(0){
                write!(f, "no values")
            } else if self.takes_exactly(1){
                write!(f, "1 value")
            } else {
                write!(f, "{} values", self.min_or_default())
            };
        }

        match (self.min, self.max) {
            (Some(min), Some(max)) => write!(f, "{} to {} values", min, max),
            (Some(min), None) => write!(f, "{} or more values", min),
            (None, Some(max)) => write!(f, "{} or less values", max),
            (None, None) => write!(f, "any number of values")
        }
    }
}

impl From<ArgCount> for RangeInclusive<usize> {
    fn from(arg_count: ArgCount) -> Self {
        arg_count.min_or_default()..=arg_count.max_or_default()
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

macro_rules! impl_value_count_from_unsigned_int {
    ($($target:ident),*) => {
        $(
            impl From<$target> for ArgCount {
                fn from(value: $target) -> Self {
                    ArgCount::exactly(value as usize)
                }
            }

            impl From<RangeInclusive<$target>> for ArgCount {
                fn from(value: RangeInclusive<$target>) -> Self {
                    let start = *value.start();
                    let end = *value.end();
                    ArgCount::new_bounded(start as usize, end as usize)
                }
            }

            impl From<Range<$target>> for ArgCount {
                fn from(value: Range<$target>) -> Self {
                    let start = value.start;
                    let end = value.end.sub(1);
                    ArgCount::new_bounded(start as usize, end as usize)
                }
            }

            impl From<RangeFrom<$target>> for ArgCount {
                fn from(value: RangeFrom<$target>) -> Self {
                    let start = value.start;
                    ArgCount::more_than(start as usize)
                }
            }

            impl From<RangeTo<$target>> for ArgCount {
                fn from(value: RangeTo<$target>) -> Self {
                    let end = value.end.sub(1);
                    ArgCount::less_than(end as usize)
                }
            }

            impl From<RangeToInclusive<$target>> for ArgCount {
                fn from(value: RangeToInclusive<$target>) -> Self {
                    let end = value.end;
                    ArgCount::less_than(end as usize)
                }
            }
        )*
    };
}

macro_rules! impl_value_count_from_signed_int {
    ($($target:ident),*) => {
        $(
            impl From<$target> for ArgCount {
                fn from(value: $target) -> Self {
                    assert!(value >= 0, "value count cannot be negative: {}", value);
                    ArgCount::exactly(value as usize)
                }
            }

            impl From<RangeInclusive<$target>> for ArgCount {
                fn from(value: RangeInclusive<$target>) -> Self {
                    let start = *value.start();
                    let end = *value.end();

                    assert!(start >= 0, "start cannot be negative");
                    assert!(end >= 0, "end cannot be negative");
                    ArgCount::new_bounded(start as usize, end as usize)
                }
            }

            impl From<Range<$target>> for ArgCount {
                fn from(value: Range<$target>) -> Self {
                    let start = value.start;
                    let end = value.end.sub(1);

                    assert!(start >= 0, "start cannot be negative");
                    assert!(end >= 0, "end cannot be negative");
                    ArgCount::new_bounded(start as usize, end as usize)
                }
            }

            impl From<RangeFrom<$target>> for ArgCount {
                fn from(value: RangeFrom<$target>) -> Self {
                    let start = value.start;
                    assert!(start >= 0, "start cannot be negative");
                    ArgCount::more_than(start as usize)
                }
            }

            impl From<RangeTo<$target>> for ArgCount {
                fn from(value: RangeTo<$target>) -> Self {
                    let end = value.end.sub(1);
                    assert!(end >= 0, "end cannot be negative");
                    ArgCount::less_than(end as usize)
                }
            }

            impl From<RangeToInclusive<$target>> for ArgCount {
                fn from(value: RangeToInclusive<$target>) -> Self {
                    let end = value.end;
                    assert!(end >= 0, "end cannot be negative");
                    ArgCount::less_than(end as usize)
                }
            }
        )*
    };
}

impl_value_count_from_unsigned_int! { u8, u16, u32, u64, u128, usize }

impl_value_count_from_signed_int! { i8, i16, i32, i64, i128, isize }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_count_test() {
        let arg_count = ArgCount::new(Some(2), Some(5));
        assert_eq!(arg_count.min_or_default(), 2);
        assert_eq!(arg_count.max_or_default(), 5);
        assert!(arg_count.takes_values());
        assert!(arg_count.takes(2));
        assert!(arg_count.takes(3));
        assert!(arg_count.takes(4));
        assert!(arg_count.takes(5));
    }

    #[test]
    fn into_value_count_test() {
        fn assert_into<A: Into<ArgCount>>(value: A, expected_min: usize, expected_max: usize) {
            let arg_count = value.into();
            let type_name = std::any::type_name::<A>();
            assert_eq!(
                arg_count.min_or_default(),
                expected_min,
                "min value - type: `{}`",
                type_name
            );
            assert_eq!(
                arg_count.max_or_default(),
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
    #[should_panic(expected = "value count cannot be negative")]
    fn into_value_count_panic_test1() {
        let _: ArgCount = (-1_i32).into();
    }

    #[test]
    #[should_panic]
    fn into_value_count_panic_test2() {
        let _: ArgCount = (1..1).into();
    }

    #[test]
    #[should_panic]
    fn into_value_count_panic_test3() {
        let _: ArgCount = (0..-2).into();
    }

    #[test]
    fn none_test() {
        let arg_count = ArgCount::zero();
        assert!(!arg_count.takes_values());
        assert!(arg_count.is_exact());
        assert_eq!(arg_count.min_or_default(), 0);
        assert_eq!(arg_count.max_or_default(), 0);
    }

    #[test]
    fn one_test() {
        let arg_count = ArgCount::one();
        assert!(arg_count.takes_values());
        assert!(arg_count.is_exact());
        assert_eq!(arg_count.min_or_default(), 1);
        assert_eq!(arg_count.max_or_default(), 1);
    }

    #[test]
    fn any_test() {
        let arg_count = ArgCount::any();
        assert!(arg_count.takes_values());
        assert!(!arg_count.is_exact());
        assert_eq!(arg_count.min_or_default(), 0);
        assert_eq!(arg_count.max_or_default(), usize::max_value());
    }

    #[test]
    fn exactly_test() {
        let arg_count = ArgCount::exactly(2);
        assert!(arg_count.takes_exactly(2));
        assert_eq!(arg_count.min_or_default(), 2);
        assert_eq!(arg_count.max_or_default(), 2);
    }

    #[test]
    fn more_than_test() {
        let arg_count = ArgCount::more_than(1);
        assert!(!arg_count.takes_exactly(1));
        assert_eq!(arg_count.min_or_default(), 1);
        assert_eq!(arg_count.max_or_default(), usize::max_value());
    }

    #[test]
    fn less_than_test() {
        let arg_count = ArgCount::less_than(5);
        assert!(!arg_count.takes_exactly(5));
        assert_eq!(arg_count.min_or_default(), 0);
        assert_eq!(arg_count.max_or_default(), 5);
    }

    #[test]
    fn contains_test() {
        let arg_count = ArgCount::new(Some(0), Some(3));
        assert!(arg_count.takes(0));
        assert!(arg_count.takes(1));
        assert!(arg_count.takes(2));
        assert!(arg_count.takes(3));
    }

    #[test]
    fn display_test() {
        assert_eq!(ArgCount::zero().to_string(), "no values");
        assert_eq!(ArgCount::new(Some(0), Some(2)).to_string(), "0 to 2 values");
        assert_eq!(ArgCount::exactly(1).to_string(), "1 value");
        assert_eq!(ArgCount::more_than(2).to_string(), "2 or more values");
        assert_eq!(ArgCount::less_than(10).to_string(), "10 or less values");
        assert_eq!(ArgCount::any().to_string(), "any number of values");
    }
}