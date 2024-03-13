pub struct SpinLock {
    locked: AtomicBool
}

impl SpinLock {
    pub const fn new() -> Self {
        Self { locked: AtomicBool::new(false) }
    }

    pub fn lock(&self) {
        while self.locked.swap(true, Acquire) {
            std::hint::spin_loop();
        }
        // 以下でもあり
        self.locked.compare_exchange_weak(false, true, Acquire, Relaxed).is_err()
    }

    pub fn unlock(&self) {
        self.locked.store(false, Release);
    }
}
