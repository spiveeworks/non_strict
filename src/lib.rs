// Library of objects for handling non-strict evaluation strategies

#![feature(unboxed_closures)]
#![feature(fn_traits)]

pub use thunk::*;
pub use lazy::*;

mod thunk;
mod lazy;


#[cfg(test)]
mod tests {
    use std::cell;
    use super::*;

    #[test]
    fn thunk_cell() {
        let called = cell::Cell::new(false);
        let val = ThunkCell::new(|| { called.set(true); 7 } );
        assert!(!called.get());
        assert_eq!(*val, 7);
        assert!(called.get());
    }
}
