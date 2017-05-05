use std::mem;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

#[derive(Debug)]
pub struct AtomicImmut<T> {
    ptr: AtomicPtr<T>,
    rwlock: SpinRwLock,
}
impl<T> AtomicImmut<T> {
    pub fn new(value: T) -> Self {
        let ptr = AtomicPtr::new(to_arc_ptr(value));
        let rwlock = SpinRwLock::new();
        AtomicImmut { ptr, rwlock }
    }
    pub fn load(&self) -> Arc<T> {
        let _guard = self.rwlock.rlock();
        let ptr = self.ptr.load(Ordering::SeqCst);
        let value = unsafe { Arc::from_raw(ptr) };
        mem::forget(value.clone());
        value
    }
    pub fn store(&self, value: T) {
        self.swap(value);
    }
    pub fn swap(&self, value: T) -> Arc<T> {
        let new = to_arc_ptr(value);
        let old = {
            let _guard = self.rwlock.wlock();
            self.ptr.swap(new, Ordering::SeqCst)
        };
        unsafe { Arc::from_raw(old) }
    }
}
unsafe impl<T> Send for AtomicImmut<T> {}
impl<T> Drop for AtomicImmut<T> {
    fn drop(&mut self) {
        let ptr = mem::replace(self.ptr.get_mut(), ptr::null_mut());
        let _ = unsafe { Arc::from_raw(ptr) };
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
    use std::sync::Arc;
    use super::*;

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
}
