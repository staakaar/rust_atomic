pub struct Mutex {
    m: Box<UnsafeCell<libc::pthread_mutex_t>>,
}

// Linux futex

#[cfg(not(target_os = "linux"))]
compile_error!("Linux only. Sorry!");

pub fn wait(a: &AtomicU32, expected: u32) {
    unsafe {
        libc::syscall(
            libc::SYS_futex, // futex system call
            a as *const AtomicU32, // target atomic variable
            libc::FUTEX_WAIT, // futex操作
            expected, // 想定される値
            std::ptr::null::<libc::timespex>() // タイムアウトはしない
        );
    }
}

pub fn wake_one(a: &AtomicU32) {
    unsafe {
        libc::syscall(
            libc::SYS_futex, // futex system call 操作対象となる32ビットアトミック変数へのポインタ
            a as *const AtomicU32, // target atomic variable 操作を表すFUTEX_WAITなどの定数
            libc::FUTEXT_WAKE, // futex operation 
            1, // wake up thread
        );
    }
}

fn main() {
    let a = AtomicU32::new(0);

    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_secs(3));
            a.store(1, Relaxed);
            wake_one(&a);
        });

        println!("Wating...");
        while a.load(Relaxed) == 0 {
            wait(&a, 0);
        }
        println!("Done!");
    });
}
