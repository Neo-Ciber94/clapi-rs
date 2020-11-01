#![allow(dead_code)]
#![allow(unused_must_use)]

mod inner_cell;
mod ext;
mod lazy;
mod once_cell;
pub use ext::*;

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
