use std::thread;

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
