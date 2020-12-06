use std::fmt::{Display, Formatter};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive, Sub};

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
    pub fn exactly(count: usize) -> Self {
        Self::new(count, count)
    }

    /// Constructs a new `ArgCount` for more than the specified number of arguments.
    #[inline]
    pub fn more_than(min: usize) -> Self {
        Self::new(min, usize::max_value())
    }

    /// Constructs a new `ArgCount` for less than the specified number of arguments.
    #[inline]
    pub fn less_than(max: usize) -> Self {
        Self::new(0, max)
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

    /// Returns `true` if this takes the provided number of arguments.
    #[inline]
    pub const fn takes(&self, count: usize) -> bool {
        count >= self.min && count <= self.max
    }

    /// Returns `true` if this takes arguments.
    #[inline]
    pub const fn takes_args(&self) -> bool {
        self.max != 0
    }

    /// Returns `true` if this takes no arguments.
    #[inline]
    pub const fn takes_no_args(&self) -> bool {
        self.min == 0 && self.max == 0
    }

    /// Returns `true` if this takes an exact number of arguments.
    #[inline]
    pub const fn is_exact(&self) -> bool {
        self.min == self.max
    }

    /// Returns `true` if this takes exactly the specified number of arguments.
    #[inline]
    pub const fn takes_exactly(&self, count: usize) -> bool {
        self.min == count && self.max == count
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

macro_rules! impl_into_for_unsigned_int {
    ($($target:ident),*) => {
        $(
            impl Into<ArgCount> for $target{
                fn into(self) -> ArgCount {
                    ArgCount::exactly(self as usize)
                }
            }

            impl Into<ArgCount> for RangeInclusive<$target>{
                fn into(self) -> ArgCount {
                    ArgCount::new(*self.start() as usize, *self.end() as usize)
                }
            }

            impl Into<ArgCount> for Range<$target>{
                fn into(self) -> ArgCount {
                    ArgCount::new(self.start as usize, self.end.sub(1) as usize)
                }
            }

            impl Into<ArgCount> for RangeFrom<$target>{
                fn into(self) -> ArgCount {
                    ArgCount::more_than(self.start as usize)
                }
            }

            impl Into<ArgCount> for RangeTo<$target>{
                fn into(self) -> ArgCount {
                    ArgCount::less_than(self.end.sub(1) as usize)
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

macro_rules! impl_into_for_signed_int {
    ($($target:ident),*) => {
        $(
            impl Into<ArgCount> for $target{
                fn into(self) -> ArgCount {
                    assert!(self >= 0, "argument count cannot be negative: {}", self);
                    ArgCount::exactly(self as usize)
                }
            }

            impl Into<ArgCount> for RangeInclusive<$target>{
                fn into(self) -> ArgCount {
                    let start = *self.start();
                    let end = *self.end();

                    assert!(start >= 0, "start cannot be negative");
                    assert!(end >= 0, "end cannot be negative");

                    ArgCount::new(start as usize, end as usize)
                }
            }

            impl Into<ArgCount> for Range<$target>{
                fn into(self) -> ArgCount {
                    let start = self.start;
                    let end = self.end.sub(1);

                    assert!(start >= 0, "start cannot be negative");
                    assert!(end >= 0, "end cannot be negative");

                    ArgCount::new(start as usize, end as usize)
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
                    let end = self.end.sub(1);
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

impl_into_for_unsigned_int! { u8, u16, u32, u64, u128, usize }

impl_into_for_signed_int! { i8, i16, i32, i64, i128, isize }

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
    fn into_arg_count_test(){
        fn assert_into<A: Into<ArgCount>>(value: A, expected_min: usize, expected_max: usize){
            let arg_count = value.into();
            let type_name = std::any::type_name::<A>();
            assert_eq!(arg_count.min(), expected_min, "min value - type: `{}`", type_name);
            assert_eq!(arg_count.max(), expected_max, "max value - type: `{}`", type_name);
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
    fn into_arg_count_panic_test1(){
        let _ : ArgCount = (-1_i32).into();
    }

    #[test]
    #[should_panic]
    fn into_arg_count_panic_test2(){
        let _ : ArgCount = (1..1).into();
    }

    #[test]
    #[should_panic]
    fn into_arg_count_panic_test3(){
        let _ : ArgCount = (0..-2).into();
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
