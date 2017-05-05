use std::mem;
use std::ptr;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicPtr, Ordering};

#[derive(Debug)]
pub struct StdAtomicImmut<T> {
    rwlock: RwLock<AtomicPtr<T>>,
}
impl<T> StdAtomicImmut<T> {
    pub fn new(value: T) -> Self {
        let ptr = AtomicPtr::new(to_arc_ptr(value));
        let rwlock = RwLock::new(ptr);
        StdAtomicImmut { rwlock }
    }
    pub fn load(&self) -> Arc<T> {
        let ptr = self.rwlock.read().unwrap();
        let raw = ptr.load(Ordering::SeqCst);
        let value = unsafe { Arc::from_raw(raw) };
        mem::forget(value.clone());
        value
    }
    pub fn store(&self, value: T) {
        self.swap(value);
    }
    pub fn swap(&self, value: T) -> Arc<T> {
        let new = to_arc_ptr(value);
        let old = {
            let ptr = self.rwlock.write().unwrap();
            ptr.swap(new, Ordering::SeqCst)
        };
        unsafe { Arc::from_raw(old) }
    }
}
impl<T> Drop for StdAtomicImmut<T> {
    fn drop(&mut self) {
        let mut ptr = self.rwlock.write().unwrap();
        let raw = mem::replace(ptr.get_mut(), ptr::null_mut());
        let _ = unsafe { Arc::from_raw(raw) };
    }
}

fn to_arc_ptr<T>(value: T) -> *mut T {
    let boxed = Arc::new(value);
    Arc::into_raw(boxed) as _
}
