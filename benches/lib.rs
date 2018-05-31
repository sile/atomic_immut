// $ rustup run nightly cargo bench
#![feature(test)]
extern crate atomic_immut;
extern crate test;

use atomic_immut::AtomicImmut;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;
use test::Bencher;

use std_atomic_immut::StdAtomicImmut;

mod std_atomic_immut;

#[bench]
fn single_thread_load(b: &mut Bencher) {
    let v = AtomicImmut::new(vec![0, 1, 2]);
    b.iter(|| {
        test::black_box(v.load());
    });
}

#[bench]
fn single_thread_load_std(b: &mut Bencher) {
    let v = StdAtomicImmut::new(vec![0, 1, 2]);
    b.iter(|| {
        test::black_box(v.load());
    });
}

#[bench]
fn multi_thread_load(b: &mut Bencher) {
    let v = Arc::new(AtomicImmut::new(vec![0, 1, 2]));
    let thread_count = 8;
    let barrier = Arc::new(Barrier::new(thread_count));
    for _ in 0..thread_count {
        let v = Arc::clone(&v);
        let barrier = Arc::clone(&barrier);
        thread::spawn(move || {
            while !v.load().is_empty() {}
            barrier.wait();
        });
    }
    thread::sleep(Duration::from_millis(10));
    b.iter(|| {
        test::black_box(v.load());
    });
    v.store(vec![]);
    barrier.wait();
    assert_eq!(Arc::strong_count(&v.load()), 2);
}

#[bench]
fn multi_thread_load_std(b: &mut Bencher) {
    let v = Arc::new(StdAtomicImmut::new(vec![0, 1, 2]));
    let thread_count = 8;
    let barrier = Arc::new(Barrier::new(thread_count));
    for _ in 0..thread_count {
        let v = Arc::clone(&v);
        let barrier = Arc::clone(&barrier);
        thread::spawn(move || {
            while !v.load().is_empty() {}
            barrier.wait();
        });
    }
    thread::sleep(Duration::from_millis(10));
    b.iter(|| {
        test::black_box(v.load());
    });
    v.store(vec![]);
    barrier.wait();
    assert_eq!(Arc::strong_count(&v.load()), 2);
}

#[bench]
fn multi_thread_store_and_load(b: &mut Bencher) {
    let v0 = Arc::new(AtomicImmut::new(vec![0, 1, 2]));
    let v1 = Arc::new(AtomicImmut::new(0));
    let thread_count = 4;
    let barrier = Arc::new(Barrier::new(thread_count));
    for _ in 0..thread_count {
        let v0 = Arc::clone(&v0);
        let v1 = Arc::clone(&v1);
        let barrier = Arc::clone(&barrier);
        thread::spawn(move || {
            while !v0.load().is_empty() {
                v1.store(1);
            }
            barrier.wait();
        });
    }
    thread::sleep(Duration::from_millis(10));
    b.iter(|| {
        test::black_box(v0.load());
        test::black_box(v1.load());
    });
    v0.store(vec![]);
    barrier.wait();
    assert_eq!(Arc::strong_count(&v0.load()), 2);
    assert_eq!(Arc::strong_count(&v1.load()), 2);
    assert_eq!(*v1.load(), 1);
}

#[bench]
fn multi_thread_store_and_load_std(b: &mut Bencher) {
    let v0 = Arc::new(StdAtomicImmut::new(vec![0, 1, 2]));
    let v1 = Arc::new(StdAtomicImmut::new(0));
    let thread_count = 4;
    let barrier = Arc::new(Barrier::new(thread_count));
    for _ in 0..thread_count {
        let v0 = Arc::clone(&v0);
        let v1 = Arc::clone(&v1);
        let barrier = Arc::clone(&barrier);
        thread::spawn(move || {
            while !v0.load().is_empty() {
                v1.store(1);
            }
            barrier.wait();
        });
    }
    thread::sleep(Duration::from_millis(10));
    b.iter(|| {
        test::black_box(v0.load());
        test::black_box(v1.load());
    });
    v0.store(vec![]);
    barrier.wait();
    assert_eq!(Arc::strong_count(&v0.load()), 2);
    assert_eq!(Arc::strong_count(&v1.load()), 2);
    assert_eq!(*v1.load(), 1);
}
