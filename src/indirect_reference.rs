use std::sync::atiomic::AtomicPtr;

fn get_data() -> &'static Data {
    static PTR: AtomicPtr<Data> = AtomicPtr::new(std::ptr::null_mut());

    let mut p = PTR.load(Acquire);

    if p.is_null() {
        p = Box::into_raw(Box::new(generate_data())); // ヒープ領域にデータを確保 into_rawで生ポインタに変換
        //他のスレッドが初期化している場合はPTRはNULLでないため、失敗となる。
        if let Err(e) = PTR.compare_exchange(std::ptr::null_mut(), p, Release, Acquire) {
            // 生ポインタをBoxへ戻してメモリリークが起こらないようにDrop
            drop(unsafe { Box::from_raw(p) });
            // 他のスレッドが格納したポインタを利用
            p = e;
        }
    }

    unsafe { &*p }
}

// L59参照