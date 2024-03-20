use std::mem::ManuallyDrop;

struct ArcData<T> {
    data_ref_count: AtomicUsize,
    alloc_ref_count: AtomicUsize,
    data: UnsafeCell<ManuallyDrop<T>>,
}

pub struct Arc<T> {
    ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Sync + Send> Send for Arc<T> {}
unsafe impl<T: Sync + Send> Sync for Weak<T> {}

pub struct Weak<T> {
    ptr: NonNull<ArcData<T>>
}

unsafe impl<T: Sync + Send> Send for Weak<T> {}
unsafe impl<T: Sync + Send> Sync for Weak<T> {}

impl<T> Arc<T> {
    pub fn new(data: T) -> Arc<T> {
        Arc {
            ptr: NonNull::from(Box::leak(Box::new(ArcData {
                alloc_ref_count: AtomicUsize::new(1),
                data_ref_count: AtomicUsize::new(1),
                data: UnsafeCell::new(ManuallyDrop::new(data)),
            }))),
        }
    }

    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.data().data.get() }
    }
}

impl<T> Weak<T> {
    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        if self.data().alloc_ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        Weak { ptr: self.ptr }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().alloc_ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
            drop(Weak { ptr: self.ptr });
        }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.data().data_ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            unsafe {
                ManuallyDrop::drop(&mut *self.data().data.get());
            }

            drop(Weak { ptr: self.ptr });
        }
    }
}

impl<T> Weak<T> {
    pub fn upgrade(&self) -> Option<Arc<T>> {
        let mut n = self.data().data_ref_count.load(Relaxed);

        loop {
            if n == 0 {
                return None;
            }
            assert!(n < usize::MAX);

            if let Err(e) = self.data().data_ref_count.compare_exchange_weak(n, n + 1, Relaxed, Relaxed) {
                n = e;
                continue;
            }

            return Some(Arc { ptr: self.ptr });
        }
    }

    pub fn downgrade(arc: &Self) -> Weak<T> {
        let mut n = arc.data().alloc_ref_count.load(Relaxed);
        loop {
            if n == usize::MAX {
                std::hint::spin_loop();
                n = arc.data().alloc_ref_count.load(Relaxed);
                continue;
            }
            assert!(n < usize::MAX - 1);
            if let Err(e) = arc.data().alloc_ref_count.compare_exchange_weak(n, n + 1, Acquire, Relaxed) {
                n = e;
                continue;
            }

            return Weak { ptr: arc.ptr };
        }
    }

    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc.data().alloc_ref_count.compare_exhange(1, usize::MAX, Acquire, Relaxed).is_err() { return None; }

        let is_unique = arc.data().data_ref_count.load(Relaxed) == 1;
        arc.data().alloc_ref_count.store(1, Release);
        if !is_unique {
            return None;
        }

        fence(Acquire);
        unsafe { Some(&mut *arc.data().data.get()) }
    }
}
