use std::borrow::Borrow;

pub trait OptionExt<T> {
    fn contains_some<U>(&self, value: U) -> bool
    where
        U: Borrow<T>,
        T: PartialEq;
}

impl<T> OptionExt<T> for Option<T> {
    #[inline]
    fn contains_some<U>(&self, value: U) -> bool
    where
        U: Borrow<T>,
        T: PartialEq,
    {
        match self {
            Some(x) => x == value.borrow(),
            None => false,
        }
    }
}
