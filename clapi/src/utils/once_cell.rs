use std::cell::UnsafeCell;

pub struct OnceCell<T>{
    value: UnsafeCell<Option<T>>
}

impl<T> OnceCell<T>{
    pub const fn new() -> Self{
        OnceCell{ value: UnsafeCell::new(None) }
    }

    pub fn has_value(&self) -> bool {
        self.get().is_some()
    }

    pub fn get(&self) -> Option<&T>{
        unsafe { &*self.value.get() }.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut T>{
        unsafe { &mut *self.value.get() }.as_mut()
    }

    pub fn get_or_init<F: FnOnce() -> T>(&self, f: F) -> &T{
        let inner = unsafe { &*self.value.get() };

        match inner {
            Some(n) => n,
            None => {
                self.set(f());
                self.get().unwrap()
            }
        }
    }

    pub fn set(&self, value: T) -> Result<(), T>{
        let inner = unsafe { &*self.value.get() };

        if inner.is_some(){
            return Err(value);
        }

        unsafe { *self.value.get() = Some(value) }
        Ok(())
    }

    pub fn take(&mut self) -> Option<T>{
        std::mem::take(self).into_inner()
    }

    pub fn into_inner(self) -> Option<T>{
        self.value.into_inner()
    }
}

impl<T> Default for OnceCell<T>{
    #[inline]
    fn default() -> Self {
        OnceCell::new()
    }
}