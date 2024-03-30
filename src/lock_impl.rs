use atomic_wait::{wait, wake_one, wake_all};

pub struct Mutex<T> {
    // 0: unlocked
    // 1: locked 他の待機スレッドはない
    // 2: locked 他に待機スレッドがある
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
        // // stateを0(アンロック)に戻す
        // self.mutex.state.store(0, Release);
        // // 待機中のスレッドがあればそのひとつを起こす
        // wake_one(&self.mutex.state);
        if self.mutex.state.swap(0, Release) == 2 {
            wake_one(&self.mutex.state);
        }
    }
}

// inline属性 インライン化により性能の向上が期待できる
// コンパイル結果のコードを呼び出し側に直接展開する方法 非常に小さい関数に関しては通常は性能が向上する
impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        // stateを１にセット
        // while self.state.swap(1, Acquire) == 1 {
        //     // すでにロック済みの場合、stateが１でなくなるまで待機する
        //     // 複数スレッドが待機していたとしてもロックできるのは1つだけなため
        //     // 複数同時に起こしたとしてもプロセッサの時間を費やすだけで１以外のスレッドはまた元に戻る
        //     // ただ起こしたスレッドがロックを獲得するとは限らない
        //     wait(&self.state, 1);
        // }

        if self.state.compare_exchange(0, 1, Acquire, Relaxed).is_err() {
            while self.state.swap(2, Acquire) != 0 {
                lock_contended(&self.state);
                // wait(&self.state, 2);
            }
        }
        MutexGuard { mutex: self }
    }

    // cold属性 コンパイラにこの関数が通常の場合(衝突がない場合)には呼ばれないことを知らせる
    fn lock_contended(state: &AtomicU32) {
        let mut spin_count = 0;

        while state.load(Relaxed) == 1 && spin_count < 100 {
            spin_count += 1;
            std::hint::spin_loop();
        }

        if state.compare_exchange(0, 1, Acquire, Relaxed).is_ok() {
            return;
        }

        while state.swap(2, Acquire) != 0 {
            wait(state, 2);
        }
    }
}
