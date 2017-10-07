#![feature(conservative_impl_trait)]
#![feature(unboxed_closures)]

use std::cell;
use std::mem;
use std::ops;


pub enum LazyOption<T, F> where F: FnOnce() -> T {
    Empty,
    Function(F),
    Result(T),
}

impl<T, F> LazyOption<T, F> where F: FnOnce() -> T {
    pub fn into_result(self) -> Option<T> {
        match self {
            LazyOption::Empty => None,
            LazyOption::Function(f) => Some(f()),
            LazyOption::Result(x) => Some(x),
        }
    }
}


pub struct LazyCell<T, F> where F: FnOnce() -> T {
    inner: cell::UnsafeCell<LazyOption<T, F>>,
}

impl<T, F> LazyCell<T, F> where F: FnOnce() -> T {
    pub fn new(func: F) -> Self {
        LazyCell{ inner: cell::UnsafeCell::new(LazyOption::Function(func)) }
    }

    pub fn promote(option: LazyOption<T, F>) -> Option<LazyCell<T, F>> {
        match option {
            LazyOption::Empty => None,
            _ => Some(LazyCell{ inner: cell::UnsafeCell::new(option) }),
        }
    }

    // in this function we use the UnsafeCell like a Cell
    pub fn evaluate(&self) {
        unsafe {
            if let &LazyOption::Function(_) = &*self.inner.get() {
                let mut dance = LazyOption::Empty;
                mem::swap(&mut *self.inner.get(), &mut dance); // like a Cell
                if let LazyOption::Function(f) = dance {
                    *self.inner.get() = LazyOption::Result(f());
                } else {
                    unreachable!();
                }
            }
        }
    }

    // borrow a LazyCell as an Fn closure
    pub fn cache_fn<'a>(&'a self) -> impl Fn() -> &'a T {
        move || &*self
    }

}

impl<T, F: FnOnce() -> T> ops::Deref for LazyCell<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        self.evaluate();
        let internal = unsafe { &*self.inner.get() };
        if let &LazyOption::Result(ref v) = internal {
            v
        } else {
            panic!("Deref on empty LazyCell");
        }
    }
}

impl<T, F: FnOnce() -> T> ops::DerefMut for LazyCell<T, F> {
    fn deref_mut(&mut self) -> &mut T {
        self.evaluate();
        let internal = unsafe { &mut *self.inner.get() };
        if let &mut LazyOption::Result(ref mut v) = internal {
            v
        } else {
            panic!("Deref on empty LazyCell");
        }
    }
}


#[cfg(test)]
mod tests {
    use std::cell;
    use super::*;

    #[test]
    fn by_name() {
        let called = cell::Cell::new(false);
        let val = LazyCell::new(|| { called.set(true); 7 } );
        assert!(!called.get());
        assert_eq!(*val, 7);
        assert!(called.get());
    }
}
