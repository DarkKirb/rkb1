use lock_api::{GuardSend, RawMutex, RawRwLock, RawRwLockDowngrade};

use self::atomic::{Atomic, AtomicBool};

pub mod atomic;

pub struct RP2040Mutex {
    locked: AtomicBool,
}

unsafe impl RawMutex for RP2040Mutex {
    const INIT: Self = Self {
        locked: AtomicBool::new(false),
    };

    type GuardMarker = GuardSend;

    fn lock(&self) {
        while !self.try_lock() {}
    }

    fn try_lock(&self) -> bool {
        !self.locked.swap(true)
    }

    unsafe fn unlock(&self) {
        self.locked.store(false)
    }

    fn is_locked(&self) -> bool {
        self.locked.load()
    }
}

pub struct RP2040RwLock {
    exclusive_lock: RP2040Mutex,
    shared_lock_count: Atomic<u16>,
}

unsafe impl RawRwLock for RP2040RwLock {
    const INIT: Self = Self {
        exclusive_lock: RP2040Mutex::INIT,
        shared_lock_count: Atomic::new(0),
    };

    type GuardMarker = GuardSend;

    fn lock_shared(&self) {
        while !self.try_lock_shared() {}
    }

    fn try_lock_shared(&self) -> bool {
        critical_section::with(|_| {
            // check if the lock is mutably borrowed
            if self.exclusive_lock.is_locked() {
                return false;
            }
            // try increasing the number of read locks, failing if we reached the max
            self.shared_lock_count
                .fetch_update(|v| {
                    if v == u16::MAX {
                        return None;
                    }
                    Some(v + 1)
                })
                .is_ok()
        })
    }

    unsafe fn unlock_shared(&self) {
        self.shared_lock_count.fetch_sub(1);
    }

    fn lock_exclusive(&self) {
        while !self.try_lock_exclusive() {}
    }

    fn try_lock_exclusive(&self) -> bool {
        critical_section::with(|_| {
            // check if the lock is already mutably borrowed
            if self.exclusive_lock.is_locked() {
                return false;
            }
            // check if the lock is already immutably borrowed
            if self.shared_lock_count.load() != 0 {
                return false;
            }
            self.exclusive_lock.try_lock()
        })
    }

    unsafe fn unlock_exclusive(&self) {
        self.exclusive_lock.unlock()
    }

    fn is_locked(&self) -> bool {
        critical_section::with(|_| self.is_locked_exclusive() || self.shared_lock_count.load() != 0)
    }

    fn is_locked_exclusive(&self) -> bool {
        self.exclusive_lock.is_locked()
    }
}

unsafe impl RawRwLockDowngrade for RP2040RwLock {
    unsafe fn downgrade(&self) {
        critical_section::with(|_| {
            self.shared_lock_count.fetch_add(1);
            self.exclusive_lock.unlock();
        });
    }
}

pub type Mutex<T> = lock_api::Mutex<RP2040Mutex, T>;
pub type RwLock<T> = lock_api::RwLock<RP2040RwLock, T>;
