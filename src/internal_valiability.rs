use std::cell::Cell;
use std::cell::RefCell;

// 【前提】マルチスレッドでは役に立たない

fn f(a: &Cell<i32>, b: &Cell<i32>) {
    let before = a.get();
    b.set(b.get() + 1);

    let after = a.get();
    if before != after {
        x();
    }
}

// Cell
fn f1(v: &Cell<Vec<i32>>) {
    let mut v2 = v.take();
    v2.push(1);
    v.set(v2);
}

// std::cell::Cell<T> Tを単にラップするだけだが共有参照を通した変更を許す

// RefCell
// std::cell::RefCell<T> 小さな実行時コストと引き換えに内容の借用を許す　
// 該当時点での借用の数を管理するカウンタも保持する シングルスレッドのみ

fn f(v: &RefCell<Vec<i32>>) {
    v.borrow_mut().push(1); // Vec内を直接変更
}