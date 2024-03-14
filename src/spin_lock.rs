use std::cell::UnsafeCell;

pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {
    const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    // pub fn lock<'a>(&'a self) -> &'a mut T {}ともかける。&selfと&mut Tの生存期間が同じであると仮定される
    // selfに対する次のunlock()呼び出しまで(別スレッドによって行われた場合も)
    fn lock(&self) -> &mut T {
        while self.locked.swap(true, Acquire) {
            std::hint::spin_loop();
        }
        unsafe { &mut *self.value.get() }
        // 以下でもあり
        // self.locked.compare_exchange_weak(false, true, Acquire, Relaxed).is_err()
    }

    unsafe fn unlock(&self) {
        self.locked.store(false, Release);
    }
}
