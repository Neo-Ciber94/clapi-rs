use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// A cell that can mutate its inner value.
#[repr(transparent)]
pub struct InnerCell<T> {
    value: UnsafeCell<T>,
}

impl<T> InnerCell<T> {
    /// Constructs a new `InnerCell`.
    #[inline]
    pub const fn new(value: T) -> Self {
        InnerCell {
            value: UnsafeCell::new(value),
        }
    }

    /// Gets a reference to the value.
    #[inline]
    pub fn get(&self) -> &T {
        unsafe { &*self.value.get() }
    }

    /// Gets a mutable reference to the value.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.value.get() }
    }

    /// Sets a new value.
    #[inline]
    pub fn set(&self, value: T) {
        unsafe { self.value.get().write(value) }
    }

    /// Updates the value with the given function.
    #[inline]
    pub fn update<F: FnOnce(T) -> T>(&self, f: F) {
        let old = unsafe { std::mem::transmute_copy(self) };
        let new = f(old);
        unsafe { self.value.get().write(new) }
    }

    /// Unwraps the value.
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

impl<T: Clone> InnerCell<T> {
    /// Sets a new value and get the old one.
    #[inline]
    pub fn replace(&self, value: T) -> T {
        let old = self.get().clone();
        self.set(value);
        old
    }
}

impl<T: Clone> Clone for InnerCell<T> {
    fn clone(&self) -> Self {
        let copy = self.get().clone();
        InnerCell::new(copy)
    }
}

impl<T: Default> Default for InnerCell<T> {
    #[inline]
    fn default() -> Self {
        InnerCell::new(T::default())
    }
}

impl<T: Debug> Debug for InnerCell<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}

impl<T: Display> Display for InnerCell<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}

impl<T: Hash> Hash for InnerCell<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state)
    }
}

impl<T: Eq + PartialEq> Eq for InnerCell<T> {}

impl<T: PartialEq> PartialEq for InnerCell<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.get().eq(other.get())
    }
}

impl<T: PartialOrd + Eq> Ord for InnerCell<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().partial_cmp(other.get()).unwrap()
    }
}

impl<T: PartialOrd> PartialOrd for InnerCell<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get().partial_cmp(other.get())
    }
}

unsafe impl<T: Send> Send for InnerCell<T> {}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn inner_cell_test1(){
        let cell = InnerCell::new(0);
        assert_eq!(*cell.get(), 0);

        cell.update(|x| x + 1);
        assert_eq!(*cell.get(), 1);
    }

    #[test]
    fn inner_cell_test2(){
        let cell = InnerCell::new(0);

        cell.set(3);
        assert_eq!(*cell.get(), 3);
    }
}
