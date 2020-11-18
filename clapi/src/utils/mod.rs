#![allow(dead_code)]
#![allow(unused_must_use)]

mod inner_cell;
mod lazy;
mod once_cell;

/// Extension methods.
mod ext;
pub use ext::*;

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
