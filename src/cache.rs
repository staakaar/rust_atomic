use std::hint::black_box;

static A: AtomicU64 = AtomicU64::new(0);

fn main() {
    black_box(&A);

    thread::spawn(|| {
        loop {
            A.store(0, Relaxed);
        }
    });

    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A.load(Relaxed));
    }
    
    println!("{:?}", start.elapsed());
}

static A: [AtomicU64; 3] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

fn main() {
    black_box(&A);

    thread::spawn(|| {
        loop {
            A[0].store(0, Relaxed); // バックグラウンドスレッドを実行しているプロセッサはAキャッシュラインが同じため、同じ影響を受ける
            A[2].store(0, Relaxed); // A[1]と同じ影響を受ける
        }
    });

    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A[1].load(Relaxed)); // 偽共有 A[0] A[2]が同じ影響を受ける
    }
    println!("{:?}", start.elapsed());
}

// アトミック変数同士を遠ざけて、別のキャッシュラインに乗るようにすれば良い
// 無関係なアトミック変数は近くに置かない

#[repr(align(64))]
struct Aligned(AtomicU64);

static A: [Aligned; 3] = [
    Aligned(AtomicU64::new(0)),
    Aligned(AtomicU64::new(0)),
    Aligned(AtomicU64::new(0)),
];

fn main() {
    black_box(&A);

    thread::spawn(|| {
        loop {
            A[0].0.store(1, Relaxed);
            A[2].0.store(1, Relaxed);
        }
    });

    let start = Instant::now();

    for _ in 0..1_000_000_000 {}

    println!("{:?}", start.elapsed());
}
