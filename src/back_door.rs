use std::ops::{Deref, DerefMut};
use std::mem;

struct BackDoorBase<T, R, F>
    where F: FnOnce(T) -> R
{
    thing: T,
    call_back: F,
}

impl<T, R, F: FnOnce(T) -> R> BackDoorBase<T, R, F> {
    fn into_result(self) -> R {
        let BackDoorBase {thing, call_back} = self;
        call_back(thing)
    }
}

pub struct BackDoor<T, R, F: FnOnce(T) -> R> {
    inner: Option<BackDoorBase<T, R, F>>,
}

fn bd_expect<V>(value: Option<V>) -> V {
    value.expect("Consumed invalid BackDoor object")
}

impl<T, R, F: FnOnce(T) -> R> Deref for BackDoor<T, R, F> {
    type Target = T;
    fn deref(&self) -> &T {
        &bd_expect(self.inner.as_ref()).thing
    }
}

impl<T, R, F: FnOnce(T) -> R> DerefMut for BackDoor<T, R, F> {
    fn deref_mut(&mut self) -> &mut T {
        &mut bd_expect(self.inner.as_mut()).thing
    }
}


impl<T, R, F: FnOnce(T) -> R> BackDoor<T, R, F> {
    pub fn new(thing: T, call_back: F) -> Self {
        BackDoor {
            inner: Some(BackDoorBase {
                thing: thing, 
                call_back: call_back,
            }),
        }
    }

    pub fn retrieve(mut self) -> (T, F) {
        let BackDoorBase{thing, call_back} = bd_expect(self.inner.take());
        (thing, call_back)
    }

    pub fn into_result(mut self) -> R {
        bd_expect(self.inner.take()).into_result()
    }
}

impl<T, R, F: FnOnce(T) -> R> Drop for BackDoor<T, R, F> {
    fn drop(&mut self) {
        self.inner   // reborrow an &mut Option<BackDoorBase>
            .take()  // empty it and extract a value
            .map(BackDoorBase::into_result);  // call and drop
    }
}

fn main() {
    let box_x = Box::new(5);
    let mut option_y = None;
    {
        let f = |box_x| {
            option_y = Some(box_x);
        };
        mem::drop(BackDoor::new(box_x, f));
    }
    if let Some(y) = option_y {
        println!("Got {}", y);
    } else {
        println!("Got nothing!");
    }
}

