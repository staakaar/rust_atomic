pub struct RwLock<T> {
    // リードロックの数を２倍 ライタが待機していた１足した値
    // リーダーが偶数の場合はロックを取得 奇数の場合はブロック
    // リーダーの数 ライトロックの場合 u32::Max
    state: AtomicU32,
    // ライタを起こす際にインクリメント
    writer_wake_counter: AtomicU32,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for RwLock<T> where T: Send + Sync {}

pub struct ReadGuard<'a, T> {
    rwlock: &'a RwLock<T>,
}

pub struct WriteGuard<'a, T> {
    rwlock: &'a RwLock<T>,
}

impl<T> RwLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            writer_wake_counter: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    pub fn read(&self) -> ReadGuard<T> {
        let mut s = self.state.load(Relaxed);
        loop {
            if s % 2 == 0 {
                assert!(s < u32::MAX - 2, "too many readers");
                match self.state.compare_exchange_weak(s, s + 2, Acquire, Relaxed) {
                    Ok(_) => return ReadGuard { rwlock: self },
                    Err(e) => s = e,
                }
            }
        }
        if s % 2 == 1 {
            wait(&self.state, s);
            s = self.state.load(Relaxed);
        }
    }

    pub fn write(&self) -> WriteGuard<T> {
        let mut s = self.state.load(Relaxed);
        loop {
            // アンロックされていたらロック
            if s <= 1 {
                match self.state.compare_exchange(s, u32::MAX, Acquire, Relaxed) {
                    Ok(_) => return WriteGuard { rwlock: self },
                    Err(e) => { s = e; continue; }
                }
            }

            // stateを奇数へ 新しいリーダーをブロックする
            if s % 2 == 0 {
                match self.state.compare_exchange(s, s + 1, Relaxed, Relaxed) {
                    Ok(_) => {}
                    Err(e) => { s = e; continue; }
                }
            }
            // まだロックされていたら待機
            let w = self.writer_wake_counter.load(Acquire);
            s = self.state.load(Relaxed);
            if s >= 2 {
                wait(&self.writer_wake_counter, w);
                s = self.state.load(Relaxed);
            }
        }
        while let Err(e) = self.state.compare_exchange(0, u32::MAX, Acquire, Relaxed).is_err() {
            let w = self.writer_wake_counter.load(Acquire);
            if self.state.load(Relaxed) != 0 {
                //RwLockがまだロックされていたら待機
                // wake通知がきていない場合
                wait(&self.writer_wake_counter, w);
            }
        }
        WriteGuard { rwlock: self }
    }
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.rwlock.value.get() }
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.rwlock.value.get() }
    }
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.rwlock.value.get() }
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        if self.rwlock.state.fetch_sub(2, Release) == 3 {
            // 3から1になった場合はRwLockがアンロック
            // 「かつ」待機中のライタがいることがわかるため起こす
            self.rwlock.writer_wake_counter.fetch_add(1, Release);
            wake_one(&self.rwlock.writer_wake_counter);
        }
    }
}

impl<T> Drop for WriteGuard<'a, T> {
    fn drop(&mut self) {
        self.rwlock.state.store(0, Release);
        self.rwlock.writer_wake_counter.fetch_add(1, Release);
        wake_one(&self.rwlock.writer_wake_counter);
        wake_all(&self.rwlock.state);
    }
}
