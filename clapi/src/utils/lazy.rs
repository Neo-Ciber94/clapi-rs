use crate::utils::once_cell::OnceCell;
use std::cell::Cell;
use std::ops::Deref;

pub struct Lazy<T, F = fn() -> T> {
    value: OnceCell<T>,
    init: Cell<Option<F>>,
}

impl<T, F> Lazy<T, F> {
    pub const fn new(init: F) -> Self {
        Lazy {
            value: OnceCell::new(),
            init: Cell::new(Some(init)),
        }
    }

    #[inline]
    pub fn is_init(&self) -> bool {
        self.value.has_value()
    }
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    fn get_or_init(&self) -> &T {
        self.value.get_or_init(|| match self.init.take() {
            Some(init) => init(),
            None => panic!("Lazy has been init"),
        })
    }
}

impl<T, F: FnOnce() -> T> Deref for Lazy<T, F> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.get_or_init()
    }
}

impl<T: Default> Default for Lazy<T> {
    fn default() -> Self {
        Lazy::new(T::default)
    }
}

unsafe impl<T> Send for Lazy<T> {}

unsafe impl<T> Sync for Lazy<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn static_lazy_test() {
        static ITEMS: Lazy<Vec<u32>> = Lazy::new(|| vec![1, 2, 3]);

        assert_eq!(ITEMS.len(), 3);
        assert_eq!(ITEMS[0], 1);
        assert_eq!(ITEMS[1], 2);
        assert_eq!(ITEMS[2], 3);
    }

    #[test]
    fn static_lazy_mut_test() {
        static ITEMS: Lazy<RefCell<Vec<u32>>> = Lazy::new(|| RefCell::new(vec![1, 2, 3]));

        assert_eq!(ITEMS.borrow().len(), 3);

        ITEMS.borrow_mut().push(4);
        ITEMS.borrow_mut().push(5);

        assert_eq!(ITEMS.borrow().len(), 5);
        assert_eq!(ITEMS.borrow()[0], 1);
        assert_eq!(ITEMS.borrow()[1], 2);
        assert_eq!(ITEMS.borrow()[2], 3);
        assert_eq!(ITEMS.borrow()[3], 4);
        assert_eq!(ITEMS.borrow()[4], 5);
    }
}
