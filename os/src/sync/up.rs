use core::cell::{RefCell, RefMut};

pub struct UpSafeCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UpSafeCell<T> {}

impl<T> UpSafeCell<T> {
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
