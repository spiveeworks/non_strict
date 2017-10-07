// This module allows for basic memoization and delayed evaluation
// it works with unboxed FnOnce monomorphisms

use std::cell;
use std::mem;
use std::ops;


// We use this enum to store our thunk/result,
// but can also use the Empty variant for swapping,
// when we want to consume the FnOnce from inside a mutable reference
pub enum ThunkEnum<T, F> where F: FnOnce() -> T {
    Empty,
    Function(F),
    Value(T),
}

impl<T, F> ThunkEnum<T, F> where F: FnOnce() -> T {
    pub fn into_value(self) -> Option<T> {
        match self {
            ThunkEnum::Empty => None,
            ThunkEnum::Function(f) => Some(f()),
            ThunkEnum::Value(x) => Some(x),
        }
    }
}

// This struct provides deref semantics for the calculated value
// It does not allow references to the function object, since
// this function object may be consumed to calculate a result
pub struct ThunkCell<T, F> where F: FnOnce() -> T {
    inner: cell::UnsafeCell<ThunkEnum<T, F>>,
}

impl<T, F> ThunkCell<T, F> where F: FnOnce() -> T {
    pub fn new(func: F) -> Self {
        ThunkCell{ inner: cell::UnsafeCell::new(ThunkEnum::Function(func)) }
    }

    pub fn value(value: T) -> Self {
        ThunkCell{ inner: cell::UnsafeCell::new(ThunkEnum::Value(value)) }
    }

    pub fn promote(option: ThunkEnum<T, F>) -> Option<Self> {
        match option {
            ThunkEnum::Empty => None,
            _ => Some(ThunkCell{ inner: cell::UnsafeCell::new(option) }),
        }
    }

    // in this function we use the UnsafeCell like a Cell
    pub fn evaluate(&self) {
        unsafe {
            if let &ThunkEnum::Function(_) = &*self.inner.get() {
                let mut dance = ThunkEnum::Empty;
                // since we checked that it was in the function state first
                mem::swap(&mut *self.inner.get(), &mut dance); // like a Cell
                if let ThunkEnum::Function(f) = dance {
                    *self.inner.get() = ThunkEnum::Value(f());
                } else {
                    unreachable!();
                }
            }
        }
    }
}

impl<T, F> ops::Deref for ThunkCell<T, F>
    where F: FnOnce() -> T,
{
    type Target = T;
    fn deref(&self) -> &T {
        self.evaluate();
        let internal = unsafe { &*self.inner.get() };
        if let &ThunkEnum::Value(ref v) = internal {
            v
        } else {
            panic!("Deref on empty ThunkCell");
        }
    }
}

impl<T, F> ops::DerefMut for ThunkCell<T, F>
    where F: FnOnce() -> T,
{
    fn deref_mut(&mut self) -> &mut T {
        self.evaluate();
        let internal = unsafe { &mut *self.inner.get() };
        if let &mut ThunkEnum::Value(ref mut v) = internal {
            v
        } else {
            panic!("Deref on empty ThunkCell");
        }
    }
}

impl<T, F> Into<T> for ThunkCell<T, F>
    where F: FnOnce() -> T,
          T: !From<ThunkMut<T, F>>,
{
    fn into(self) -> T {
        unsafe {
            match self.inner.into_inner() {
                ThunkEnum::Function(f) => f(),
                ThunkEnum::Value(value) => value,
                ThunkEnum::Empty => panic!("Unwrapped empty ThunkCell"),
            }
        }
    }
}


// like ThunkCell, but with normal mut semantics
// using this may help the optimizer rearrange the code it is used in.
// removing interior mutability defeats the purpose of a lot of use-cases though
struct ThunkMut<T, F> where F: FnOnce() -> T {
    inner: ThunkEnum<T, F>,
}

impl<T, F> ThunkMut<T, F> where F: FnOnce() -> T {
    pub fn new(func: F) -> Self {
        ThunkMut{ inner: ThunkEnum::Function(func) }
    }

    pub fn value(value: T) -> Self {
        ThunkMut{ inner: ThunkEnum::Value(value) }
    }

    pub fn promote(option: ThunkEnum<T, F>) -> Option<Self> {
        match option {
            ThunkEnum::Empty => None,
            _ => Some(ThunkMut{ inner: option }),
        }
    }

    pub fn evaluate(&mut self) {
        let mut dance = ThunkEnum::Empty;
        mem::swap(&mut self.inner, &mut dance);
        self.inner = match dance {
            ThunkEnum::Function(f) => ThunkEnum::Value(f()),
            value => value,
        }
    }
}

impl<T, F> Into<T> for ThunkMut<T, F>
    where F: FnOnce() -> T,
          T: !From<ThunkMut<T, F>>,
{
    fn into(self) -> T {
        match self.inner {
            ThunkEnum::Function(f) => f(),
            ThunkEnum::Value(value) => value,
            ThunkEnum::Empty => panic!("Unwrapped empty ThunkMut"),
        }
    }
}



impl<T, F> From<ThunkMut<T, F>> for ThunkCell<T, F>
    where F: FnOnce() -> T,
{
    fn from(as_mut: ThunkMut<T, F>) -> Self {
        ThunkCell {
            inner: cell::UnsafeCell::new(as_mut.inner),
        }
    }
}

impl<T, F> From<ThunkCell<T, F>> for ThunkMut<T, F>
    where F: FnOnce() -> T,
{
    fn from(as_cell: ThunkCell<T, F>) -> Self {
        unsafe {
            ThunkMut {
                inner: as_cell.inner.into_inner(),
            }
        }
    }
}


