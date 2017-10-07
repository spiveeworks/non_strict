

struct Memo<Args, T, Call>
    where Args: Hash,
          Call: MemoFn<Args=Args, Value=T>,
{
    call: Call,
    items: HashMap<Args, Rc<ThunkCell<T, MemoThunk<Args, T, Call>>>>,
}

struct MemoThunk<'f, Args, T, F>
    where F: Fn(Args) -> T + 'f,
{
    func: &'f F,
    args: Args,
}


impl FnOnce for MemoThunk<Args, T, F> {
    type Output = T;
    extern "rust-call" fn call_once(self, args: ()) -> T {
        self.func(self.args)
    }
}

#[test]
fn memoize() {
    let memo = RefCell::new(None);
    let fact = |memo, n| n * memo.borrow_mut().unwrap().unwrap()[n - 1];
    *memo.borrow_mut().unwrap() = Some(Memo{fact, HashMap::new()});
    assert_eq!(fact(5), 120);
}
