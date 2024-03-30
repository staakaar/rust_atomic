fn main() {
    let m = Mutex::new(0);
    std::hint::black_box(&m);
    let start = Instant::now();
    for _ in 0..5_000_000 {
        *m.lock() += 1;
    }

    let duration = start.elapsed();
    println!("locked {} times in {:?}", *m.lock(), duration);
}
/** 
 * メモ
 * AMDプロセッサ
 * 最適化前のMutex　４００ミリ秒 最適化後のMutex 40ミリ秒
 * Itenlプロセッサ
 * 最適化前のMutex 1800ミリ秒 最適化後のMutex 60ミリ秒
 * MacOS
 * いずれも50ミリ秒
 */

// 4つのスレッドが並行して１つのMutexに対してロックとアンロックを数百万回繰り返す場合
fn main() {
    let m = Mutex::new(0);
    std::hint::black_box(&m);
    let start = Instant::now();
    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                for _ in 0..5_000_000 {
                    *m.lock() += 1;
                }
            });
        }
    });

    let duration = start.elapsed();
    println!("locked {} times in {:?}", *m.lock(), duration);
}

/** 
 * Intelプロセッサ
 * スピンなし 900ミリ秒 スピンあり 750ミリ秒
 * AMDプロセッサ
 * スピンなし 650ミリ秒 スピンあり 800ミリ秒
 */

// 条件変数
pub struct Condvar {
    counter: AtomicU32,
    num_waiters: AtomicUsize,
}

impl Condvar {
    pub const fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
            //１スレッド１バイト消費の場合、同時に存在するスレッドの数を数えるのに十分
            num_waiters: AtomicUsize::new(0),
        }
    }

    pub fn notify_one(&self) {
        if self.num_waiters.load(Relaxed) > 0 {
            self.counter.fetch_add(1, Relaxed);
            wake_one(&self.counter);
        }
    }

    pub fn notify_all(&self) {
        if self.num_waiters.load(Relaxed) > 0 {
            self.counter.fetch_add(1, Relaxed);
            wake_all(&self.counter);
        }
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        self.num_waiters.fetch_add(1, Relaxed);

        let counter_value = self.counter.load(Relaxed);

        let mutex = guard.mutex;
        drop(guard);

        // カウンタ値がアンロックする前から変更されていない場合にだけ待機する。
        wait(&self.counter, counter_value);

        self.num_waiters.fetch_sub(1, Relaxed);

        mutex.lock()
    }
}

#[test]
fn test_condvar() {
    let mutex = Mutex::new(0);
    let condvar = Condvar::new();

    let mut wakeups = 0;

    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_secs(1));
            *mutex.lock() = 123;
            condvar.notify_one();
        });

        let mut m = mutex.lock();
        while *m < 100 {
            m = condvar.wait(m);
            wakeups += 1;
        }

        assert_eq!(*m, 123);
    });

    assert!(wakeups < 10);
}