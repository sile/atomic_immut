//! Atomic immutable value.
//!
//! # Examples
//!
//! ```
//! use std::sync::Arc;
//! use std::thread;
//! use atomic_immut::AtomicImmut;
//!
//! let v = Arc::new(AtomicImmut::new(vec![0]));
//! {
//!     let v = v.clone();
//!     thread::spawn(move || {
//!                       let mut new = (&*v.load()).clone(); // Loads the immutable reference
//!                       new.push(1);
//!                       v.store(new); // Replaces the existing value
//!                   });
//! }
//! while v.load().len() == 1 {}
//! assert_eq!(&*v.load(), &vec![0, 1]);
//! ```
#![warn(missing_docs)]
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::Arc;

/// A thread-safe pointer for immutable value.
///
/// This is a thin container. Each `AtomicImmut` instance has an immutable value.
/// After the `AtomicImmut` instance is created,
/// it is not possible to modify a part of the contained value.
/// But you can replace the value entirely with another value.
///
/// `AtomicImmut` is useful for sharing rarely updated and
/// complex (e.g., hashmap) data structures between threads.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use std::sync::Arc;
/// use std::thread;
/// use atomic_immut::AtomicImmut;
///
/// let mut map = HashMap::new();
/// map.insert("foo", 0);
///
/// let v = Arc::new(AtomicImmut::new(map));
/// {
///     let v = v.clone();
///     thread::spawn(move || {
///                       let mut new = (&*v.load()).clone();
///                       new.insert("bar", 1);
///                       v.store(new);
///                   });
/// }
/// while v.load().len() == 1 {}
/// assert_eq!(v.load().get("foo"), Some(&0));
/// assert_eq!(v.load().get("bar"), Some(&1));
/// ```
#[derive(Debug)]
pub struct AtomicImmut<T> {
    ptr: AtomicPtr<T>,
    rwlock: SpinRwLock,
}
impl<T> AtomicImmut<T> {
    /// Makes a new `AtomicImmut` instance.
    pub fn new(value: T) -> Self {
        let ptr = AtomicPtr::new(to_arc_ptr(value));
        let rwlock = SpinRwLock::new();
        AtomicImmut { ptr, rwlock }
    }

    /// Loads the value from this pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomic_immut::AtomicImmut;
    ///
    /// let value = AtomicImmut::new(5);
    /// assert_eq!(*value.load(), 5);
    /// ```
    pub fn load(&self) -> Arc<T> {
        let _guard = self.rwlock.rlock();
        let ptr = self.ptr.load(Ordering::SeqCst);
        let value = unsafe { Arc::from_raw(ptr) };
        mem::forget(Arc::clone(&value));
        value
    }

    /// Stores a value into this pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomic_immut::AtomicImmut;
    ///
    /// let value = AtomicImmut::new(5);
    /// assert_eq!(*value.load(), 5);
    ///
    /// value.store(1);
    /// assert_eq!(*value.load(), 1);
    /// ```
    pub fn store(&self, value: T) {
        self.swap(value);
    }

    /// Updates the value of this pointer by calling `f` on the value to get a new value.
    ///
    /// The function `f` may be called more than once when there is a conflict with other threads.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomic_immut::AtomicImmut;
    ///
    /// let value = AtomicImmut::new(5);
    /// assert_eq!(*value.load(), 5);
    ///
    /// value.update(|v| *v * 2);
    /// assert_eq!(*value.load(), 10);
    /// ```
    pub fn update<F>(&self, f: F)
    where
        F: for<'a> Fn(&'a T) -> T,
    {
        loop {
            let old = self.load();

            let new = to_arc_ptr(f(&old));
            let old = Arc::into_raw(old) as *mut _;
            unsafe { Arc::from_raw(old) };

            let _guard = self.rwlock.wlock();
            if self.ptr.compare_and_swap(old, new, Ordering::SeqCst) == old {
                unsafe { Arc::from_raw(old) };
                break;
            } else {
                unsafe { Arc::from_raw(new) };
            }
        }
    }

    /// Stores a value into this pointer, returning the old value.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomic_immut::AtomicImmut;
    ///
    /// let value = AtomicImmut::new(5);
    /// assert_eq!(*value.load(), 5);
    ///
    /// let old = value.swap(1);
    /// assert_eq!(*value.load(), 1);
    /// assert_eq!(*old, 5);
    /// ```
    pub fn swap(&self, value: T) -> Arc<T> {
        let new = to_arc_ptr(value);
        let old = {
            let _guard = self.rwlock.wlock();
            self.ptr.swap(new, Ordering::SeqCst)
        };
        unsafe { Arc::from_raw(old) }
    }
}
unsafe impl<T: Send + Sync> Send for AtomicImmut<T> {}
unsafe impl<T: Send + Sync> Sync for AtomicImmut<T> {}
impl<T> Drop for AtomicImmut<T> {
    fn drop(&mut self) {
        let ptr = mem::replace(self.ptr.get_mut(), ptr::null_mut());
        let _ = unsafe { Arc::from_raw(ptr) };
    }
}
impl<T: Default> Default for AtomicImmut<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[derive(Debug)]
struct SpinRwLock(AtomicUsize);
impl SpinRwLock {
    fn new() -> Self {
        SpinRwLock(AtomicUsize::new(0))
    }
    fn rlock(&self) -> ReadGuard {
        let old = self.0.fetch_add(1, Ordering::SeqCst);
        let mut writers = old >> reader_bits();
        while writers != 0 {
            writers = self.0.load(Ordering::SeqCst) >> reader_bits();
        }
        ReadGuard(self)
    }
    fn runlock(&self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
    fn wlock(&self) -> WriteGuard {
        while self.0.fetch_add(1 << reader_bits(), Ordering::SeqCst) != 0 {
            self.0.fetch_sub(1 << reader_bits(), Ordering::SeqCst);
            while self.0.load(Ordering::SeqCst) != 0 {}
        }
        WriteGuard(self)
    }
    fn wunlock(&self) {
        self.0.fetch_sub(1 << reader_bits(), Ordering::SeqCst);
    }
}

#[derive(Debug)]
struct ReadGuard<'a>(&'a SpinRwLock);
impl<'a> Drop for ReadGuard<'a> {
    fn drop(&mut self) {
        self.0.runlock();
    }
}

#[derive(Debug)]
struct WriteGuard<'a>(&'a SpinRwLock);
impl<'a> Drop for WriteGuard<'a> {
    fn drop(&mut self) {
        self.0.wunlock();
    }
}

fn to_arc_ptr<T>(value: T) -> *mut T {
    let boxed = Arc::new(value);
    Arc::into_raw(boxed) as _
}

#[inline]
fn reader_bits() -> usize {
    mem::size_of::<usize>() * 8 / 2
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn it_works() {
        let v = AtomicImmut::new(vec![0, 1, 2]);
        assert_eq!(&*v.load(), &vec![0, 1, 2]);
        assert_eq!(Arc::strong_count(&v.load()), 2);

        let old = v.swap(vec![0]);
        assert_eq!(&*v.load(), &vec![0]);
        assert_eq!(Arc::strong_count(&v.load()), 2);

        assert_eq!(&*old, &vec![0, 1, 2]);
        assert_eq!(Arc::strong_count(&old), 1);
    }

    #[test]
    fn multithread_test() {
        let v = Arc::new(AtomicImmut::new(vec![0, 1, 2]));
        let thread_count = 32;
        let barrier = Arc::new(Barrier::new(thread_count));
        for _ in 0..thread_count {
            let v = v.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                while !v.load().is_empty() {
                    thread::sleep(Duration::from_millis(1));
                }
                barrier.wait();
            });
        }
        thread::sleep(Duration::from_millis(10));

        v.store(vec![]);
        barrier.wait();
        assert!(v.load().is_empty());
        assert_eq!(Arc::strong_count(&v.load()), 2);
    }

    #[test]
    fn update_works() {
        let v = AtomicImmut::new(vec![0, 1, 2]);
        assert_eq!(&*v.load(), &vec![0, 1, 2]);
        assert_eq!(Arc::strong_count(&v.load()), 2);

        v.update(|_| vec![0]);
        assert_eq!(&*v.load(), &vec![0]);
        assert_eq!(Arc::strong_count(&v.load()), 2);
    }
}
