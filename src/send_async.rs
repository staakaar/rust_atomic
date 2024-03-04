use std::marker::PhantomData;

// PhantomData<Cell<()>>はCell<()>として扱われて、Syncではない Sendではある
struct X {
    handle: i32,
    _not_sync: PhantomData<Cell<()>>,
}

struct Y {
    p: *mut i32,
}

unsafe impl Send for Y {}
unsafe impl Sync for Y {}

fn main() {
    let a = Rc::new(123);
    thread::spawn(move || {
        dbg!(a);
    });
}

// RcはSendを実装していない クロージャーがSendとなるのはすべての値がSendである場合に限る。Sendでない場合はコンパイラが検知してくれる。