fn main() {
    let locked = AtomicBool::new(false);
    let counter = AtomicUsize::new(0);

    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| for _ in 0..1_000_000 {
                // メモリオーダリングが間違っている
                while locked.swap(true, Relaxed) {}
                compiler_fence(Acquire);

                // ロックを保持したまま、非アトミックにカウンタをインクリメント
                let old = counter.load(Relaxed);
                let new = old + 1;
                counter.store(new, Relaxed);

                // ロックを解放
                compiler_fence(Release);
                locked.store(false, Relaxed);
            });
        }
    });

    println!("{}", counter.into_inner());
}