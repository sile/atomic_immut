atomic_immut
============

[![Crates.io: atomic_immut](http://meritbadge.herokuapp.com/atomic_immut)](https://crates.io/crates/atomic_immut)
[![Documentation](https://docs.rs/atomic_immut/badge.svg)](https://docs.rs/atomic_immut)
[![Build Status](https://travis-ci.org/sile/atomic_immut.svg?branch=master)](https://travis-ci.org/sile/atomic_immut)
[![Code Coverage](https://codecov.io/gh/sile/atomic_immut/branch/master/graph/badge.svg)](https://codecov.io/gh/sile/atomic_immut/branch/master)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Atomic immutable value for Rust.

[Documentation](https://docs.rs/atomic_immut)


Benchmark
----------

```console
$ cargo +nightly bench

running 6 tests
test multi_thread_load               ... bench:         576 ns/iter (+/- 510)
test multi_thread_load_std           ... bench:       1,113 ns/iter (+/- 1,130)
test multi_thread_store_and_load     ... bench:         483 ns/iter (+/- 74)
test multi_thread_store_and_load_std ... bench:      27,897 ns/iter (+/- 6,171)
test single_thread_load              ... bench:          22 ns/iter (+/- 1)
test single_thread_load_std          ... bench:          41 ns/iter (+/- 0)
```
