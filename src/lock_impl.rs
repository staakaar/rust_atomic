use atomic_wait::{wait, wake_one, wake_all};

pub struct Mutex<T> {
    // 0: unlocked
    // 1: locked
    state: AtomicU32,
    value: UnsafeCell<T>,
}

// 安全なロックインターフェース
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

unsafe impl<T> Sync for MutexGuard<'_, T> where T: Sync {}
// スレッド間共有できるように
unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // stateを0(アンロック)に戻す
        self.mutex.state.store(0, Release);
        // 待機中のスレッドがあればそのひとつを起こす
        wake_one(&self.mutex.state);
    }
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        // stateを１にセット
        while self.state.swap(1, Acquire) == 1 {
            // すでにロック済みの場合、stateが１でなくなるまで待機する
            // 複数スレッドが待機していたとしてもロックできるのは1つだけなため
            // 複数同時に起こしたとしてもプロセッサの時間を費やすだけで１以外のスレッドはまた元に戻る
            // ただ起こしたスレッドがロックを獲得するとは限らない
            wait(&self.state, 1);
        }
        MutexGuard { mutex: self }
    }
}
