
use std::rc::Rc;
use ThunkCell;

struct LazyList<T, Fh, Ft>
    where Fh: FnOnce() -> T,
          Ft: FnOnce() -> LazyList<T, Fh, Ft>
{
    head: Rc<ThunkCell<T, Fh>>,
    tail: Rc<ThunkCell<LazyList<T, Fh, Ft>, Ft>>,
}

struct Repeat<T, F>(Rc<ThunkCell<T, F>>) where F: FnOnce() -> T;

impl<T, F> FnOnce<()> for Repeat<T, F>
    where F: FnOnce() -> T
{
    type Output = LazyList<T, F, Repeat<T, F>>;
    extern "rust-call" fn call_once(self, args: ()) -> Self::Output {
        LazyList{
            head: Rc::clone(&self.0),
            tail: Rc::new(ThunkCell::new(self)),
        }
    }
}

#[test]
fn lazy_list() {
    use std::cell::Cell;
    let mut counter = Cell::new(0);
    let five = || { counter.set(1+counter.get()); 5 };
    let fives = Repeat(Rc::new(ThunkCell::new(five)))();
    assert_eq!(counter.get(), 0);
    assert_eq!(**fives.head, 5);
    assert_eq!(counter.get(), 1);
    assert_eq!(**fives.tail.tail.tail.tail.tail.tail.head, 5);
    assert_eq!(counter.get(), 1);
}
