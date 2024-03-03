fn main() {
    let a = 1;
    let mut b = 1;
    f(&a, &mut b);
}

fn f(a: &i32, b: &mut i32) {
    let before = *a;
    *b += 1;

    let after = *a;
    if before != after {
        x();
    }
}

fn x() {}

// unsafe
// コンパイラがコードが安全であることを検証しないという意味
// 不健全 コードがルルーを破っている場合