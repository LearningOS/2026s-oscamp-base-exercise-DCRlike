//! # Spin Lock
//!
//! In this exercise, you will implement a basic spin lock.
//! Spin locks are one of the most fundamental synchronization primitives in OS kernels.
//!
//! ## Key Concepts
//! - Spin locks use busy-waiting to acquire the lock
//! - `AtomicBool`'s `compare_exchange` to implement lock acquisition
//! - `core::hint::spin_loop` to reduce CPU power consumption
//! - `UnsafeCell` provides interior mutability

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// Basic spin lock
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for SpinLock<T> {}
unsafe impl<T: Send> Send for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquire lock, returning a mutable reference to inner data.
    ///
    /// TODO: Use compare_exchange to spin until lock is acquired
    /// 1. In a loop, try to change locked from false to true
    /// 2. Success uses Acquire ordering, failure uses Relaxed
    /// 3. On failure call `core::hint::spin_loop()` to hint CPU
    /// 4. On success return `&mut *self.data.get()`
    ///
    /// # Safety
    /// Caller must ensure `unlock` is called after using the data.
    #[deny(clippy::mut_from_ref)]
    pub fn lock(&mut self) -> &mut T {
        unsafe {
            while self
                .locked
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                while self.locked.load(Ordering::Relaxed) {
                    core::hint::spin_loop();
                }
            }
            // SAFETY: We just acquired the lock, so no other thread holds a reference.
            // The caller is responsible for calling unlock() before this reference is used again.
            &mut (*self.data.get())
        }
    }

    /// Release lock.
    ///
    /// TODO: Set locked to false (using Release ordering)
    pub fn unlock(&self) {
        // TODO
        self.locked.store(false, Ordering::Release);
    }

    /// Try to acquire lock without spinning.
    /// Returns Some(&mut T) on success, None if lock is busy.
    #[deny(clippy::mut_from_ref)]
    pub fn try_lock(&mut self) -> Option<&mut T> {
        unsafe {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            // SAFETY: compare_exchange succeeded, meaning we are the sole lock holder.
            .map(|_|  &mut (*self.data.get()) )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_lock_unlock() {
        let lock = SpinLock::new(0u32);
        {
            let data = lock.lock();
            *data = 42;
            lock.unlock();
        }
        let data = lock.lock();
        assert_eq!(*data, 42);
        lock.unlock();
    }

    #[test]
    fn test_try_lock() {
        let lock = SpinLock::new(0u32);
        assert!(lock.try_lock().is_some());
        lock.unlock();
    }

    #[test]
    fn test_concurrent_counter() {
        let lock = Arc::new(SpinLock::new(0u64));
        let mut handles = vec![];

        for _ in 0..10 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    let data = l.lock();
                    *data += 1;
                    l.unlock();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let data = lock.lock();
        assert_eq!(*data, 10000);
        lock.unlock();
    }

    #[test]
    fn test_lock_protects_data() {
        let lock = Arc::new(SpinLock::new(Vec::new()));
        let mut handles = vec![];

        for i in 0..5 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                let data = l.lock();
                data.push(i);
                l.unlock();
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let data = lock.lock();
        let mut sorted = data.clone();
        sorted.sort();
        assert_eq!(sorted, vec![0, 1, 2, 3, 4]);
        lock.unlock();
    }
}
