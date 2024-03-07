use std::sync::atomic::AtomicI32;

fn main() {
    let a = AtomicI32::new(100);
    let b = a.fetch_add(23, Relaxed); // 100から123に増加させる 古い値100を返す
    let c = a.load(Relaxed); // 123が取り出される

    assert_eq!(b, 100);
    assert_eq!(c, 123);
}
