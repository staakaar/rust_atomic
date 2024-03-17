use std::ptr::NonNull;

pub struct Arc<T> {
    ptr: NonNull<ArcData<T>>,
}

struct ArcData<T> {
    ref_count: AtomicUsize,
    data: T,
}
// 参照、Boxに関してはコンパイラが自動的にSend、Syncを判断してくれる
// 生ポインタ、NonNullに関しては明示的に指示する必要があり

unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

// Arc<T>::new ArcData<T>を新たにメモリ上に確保
// Box::new メモリ領域の確保
// Box::leak　排他的な所有権を放棄
// NonNull::fromポインタ変換
impl<T> Arc<T> {
    pub fn new(data: T) -> Arc<T> {
        Arc {
            ptr: NonNull::from(Box::leak(Box::new(ArcData {
                ref_count: AtomicUsize::new(1),
                data,
            }))),
        }
    }

    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc.data().ref_count.load(Relaxed) == 1 {
            // Arcはひとつしかないため、データにアクセスできない
            fence(Acquire);
            unsafe { Some(&mut arc.ptr.as_mut().data) }
        } else {
            None
        }
    }
}

// Derefトレイト Arc<T>がTへの参照のように振る舞う
// DerefMutはArc<T>は共有所有を表しているため無条件に&mut Tを与えることはできない
impl<T> Deref for Arc<T> {
    type target = T;

    fn deref(&self) -> &T {
        &self.data().data
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        // すべてのスレッドが最低でも２バイトのメモリを必要とすると仮定すると、usize::MAX / 2個のスレッドが同時に存在することができないから
        if self.data().ref_count.fetch_add(1, Relaxed) > usize::Max / 2 {
            std::process::abort();
        }

        Arc {
            ptr: self.ptr,
        }
    }
}

// Box::from_rawを使用してそのメモリ領域への排他所有権を取得してdrop
impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.data().ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}

#[test]
fn test() {
    static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);

    struct DetectDrop;

    impl Drop for DetectDrop {
        fn drop(&mut self) {
            NUM_DROPS.fetch_add(1, Relaxed);
        }
    }

    let x = Arc::new(("hello", DetectDrop));
    let y = x.clone();

    let t = std::thread::spawn(move || {
        assert_eq!(x.0, "hello");
    });

    assert_eq!(y.0, "hello");

    t.join().unwrap();

    // xはここまででドロップされている

    assert_eq!(NUM_DROPS.load(Relaxed), 0);

    drop(y);

    assert_eq!(NUM_DROPS.load(Relaxed), 1);
}
