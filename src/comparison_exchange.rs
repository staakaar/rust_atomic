use std::sync::atomic::AtomicI32;

impl AtomicI32 {
    pub fn compare_exchange(&self, expected: i32, new: i32) -> Result<i32, i32> {
        let v = self.load();
        if v == expected {
            self.store(new);
            Ok(v)
        } else {
            Err(v)
        }
    }

    fn increment(a: &AtomicU32) {
        let mut current = a.load(Relaxed);
        loop {
            let mew = current + 1;
            match a.compare_exchange(current, new, Relaxed, Relaxed) {
                Ok(_) => return,
                Err(v) => current = v,
            }
        }
    }

    // ID発行においてオーバーフローしないよう上限付きのアトミックな加算
    fn allocate_new_id() -> u32 {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        let mut id = NEXT_ID.load(Relaxed);

        loop {
            assert!(id < 1000, "too many IDs!");
            match NEXT_ID.compare_exchange_weak(id, id + 1, Relaxed, Relaxed) {
                Ok(_) => return id,
                Err(v) => id = v,
            }
        }
    }

    // 必ずいつでも同じ一意な値が取得できないといけないケース
    fn get_key() -> u64 {
        static KEY: AtomicU64 = AtomicU64::new(0);
        let key = KEY.load(Relaxed);
        if key == 0 {
            let new_key = generate_random_key();
            match KEY.compare_exchange(0, new_key, Relaxed, Relaxed) {
                Ok(_) => new_key,
                Err(k) => k,
            }
        } else {
            key
        }
    }
}