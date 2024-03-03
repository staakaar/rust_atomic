mod one_four_borrowing_data_conflict;

use std::thread;
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    let t1 = thread::spawn(f);
    let t2 = thread::spawn(f);

    println!("Hello, from the main thread!");

    t1.join().unwrap(); // スレッドが実行を終了するまで待つ
    t2.join().unwrap();

    let numbers = Vec::from_iter(0..=1000);
    let t = thread::spawn(move || {
        let len = numbers.len();
        let sum = numbers.into_iter().sum::<usize>();
        sum / len
    });

    let average = t.join().unwrap();

    println!("average: {average}");

    let numbers = vec![1, 2, 3];
    thread::scope(|s| {
        s.spawn(|| {
            println!("length: {}", numbers.len());
            // numbers.push(1); error
        });
        s.spawn(|| {
            // numbers.push(2); error
            for n in &numbers {
                println!("{n}");
            }
        });
    });

    static X: [i32; 3] = [1, 2, 3]; // スレッド間で共有されている
    thread:: spawn(|| dbg!(&X));
    thread::spawn(|| dbg!(&X));

    // スレッドを共有するために所有権を解放
    let x: &'static [i32; 3] = Box::leak(Box::new([1, 2, 3])); // 'staticライフタイムでここからプログラム終了まで値が存在
    thread::spawn(move || dbg!(x));
    thread::spawn(move || dbg!(x));

    // 所有権の共有
    // 変数の所有権の数を管理し所有者がいなくなった時に値をドロップ
    // 参照カウント・・・変数が所持している所有権の数
    let a = Rc::new([1, 2, 3]); //別スレッドに送るのは同時に参照カウンタを変更しようとするためunsafe
    let b = a.clone(); // Boxのクローンは新しいコピーがメモリ上に作成される

    assert_eq!(a.as_ptr(), b.as_ptr());

    // 複数スレッドで使用しても安全 Arcを使用して値を共有している場合は、同じ参照カウントを持つ変数は同じ名前にする
    let a = Arc::new([1, 2, 3]);
    let b = a.clone();

    thread::spawn(move || dbg!(a));
    thread::spawn(move || dbg!(b));


}

fn f() {
    println!("Hello from another thread!");

    // スレッド識別子の取得
    let id = thread::current().id();
    println!("This id my thread id: {id:?}");
}

// #[allow(unused)]
// use std::{
//     cell::{Cell, RefCell, UnsafeCell},
//     collections::VecDeque, marker::PhantomData,
//     mem::{ManuallyDrop, MaybeUninit},
//     ops::{Deref, DerefMut},
//     ptr::NonNull,
//     rc::Rc,
//     sync::{*, atomic::{*, Ordering::*}},
//     thread::{self, Thread}, 
// };
