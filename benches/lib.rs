// $ rustup run nightly cargo bench
#![feature(test)]
extern crate atomic_immut;
extern crate test;

use std::sync::{Arc, Barrier};
use std::thread;
use atomic_immut::AtomicImmut;
use test::Bencher;

use std_atomic_immut::StdAtomicImmut;

mod std_atomic_immut;

#[bench]
fn single_thread_load(b: &mut Bencher) {
    let v = AtomicImmut::new(vec![0, 1, 2]);
    b.iter(|| { test::black_box(v.load()); });
}

#[bench]
fn single_thread_load_std(b: &mut Bencher) {
    let v = StdAtomicImmut::new(vec![0, 1, 2]);
    b.iter(|| { test::black_box(v.load()); });
}

#[bench]
fn multi_thread_load(b: &mut Bencher) {
    let v = Arc::new(AtomicImmut::new(vec![0, 1, 2]));
    let thread_count = 16;
    let barrier = Arc::new(Barrier::new(thread_count));
    for _ in 0..thread_count {
        let v = v.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
                          while !v.load().is_empty() {
                              //thread::sleep_ms(1);
                          }
                          barrier.wait();
                      });
    }
    thread::sleep_ms(10);
    b.iter(|| { test::black_box(v.load()); });
    v.store(vec![]);
    barrier.wait();
    assert!(v.load().is_empty());
    assert_eq!(Arc::strong_count(&v.load()), 2);
}

#[bench]
fn multi_thread_load_std(b: &mut Bencher) {
    let v = Arc::new(StdAtomicImmut::new(vec![0, 1, 2]));
    let thread_count = 16;
    let barrier = Arc::new(Barrier::new(thread_count));
    for _ in 0..thread_count {
        let v = v.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
                          while !v.load().is_empty() {
                              //thread::sleep_ms(1);
                          }
                          barrier.wait();
                      });
    }
    thread::sleep_ms(10);
    b.iter(|| { test::black_box(v.load()); });
    v.store(vec![]);
    barrier.wait();
    assert!(v.load().is_empty());
    assert_eq!(Arc::strong_count(&v.load()), 2);
}
