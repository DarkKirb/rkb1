//! Atomics for the m0+

use core::{
    cell::Cell,
    ops::{Add, BitAnd, BitOr, BitXor, Not, Sub},
};

#[derive(Default, Debug)]
pub struct Atomic<T: Copy> {
    v: Cell<T>,
}

unsafe impl<T: Copy> Send for Atomic<T> {}
unsafe impl<T: Copy> Sync for Atomic<T> {}

impl<T: Copy> Atomic<T> {
    pub const fn new(v: T) -> Self {
        Self { v: Cell::new(v) }
    }
    pub fn get_mut(&mut self) -> &mut T {
        self.v.get_mut()
    }
    pub fn into_inner(self) -> T {
        self.v.into_inner()
    }
    pub fn load(&self) -> T {
        critical_section::with(|_| self.v.get())
    }
    pub fn store(&self, v: T) {
        critical_section::with(|_| self.v.set(v))
    }
    pub fn swap(&self, v: T) -> T {
        critical_section::with(|_| {
            let old = self.v.get();
            self.v.set(v);
            old
        })
    }
    pub fn compare_exchange(&self, current: T, new: T) -> Result<T, T>
    where
        T: PartialEq,
    {
        critical_section::with(|_| {
            let old = self.v.get();
            if old != current {
                Err(old)
            } else {
                self.v.set(new);
                Ok(current)
            }
        })
    }
    pub fn fetch_add(&self, v: T) -> T
    where
        T: Add<Output = T>,
    {
        critical_section::with(|_| {
            let curr = self.v.get();
            self.v.set(curr + v);
            curr
        })
    }
    pub fn fetch_sub(&self, v: T) -> T
    where
        T: Sub<Output = T>,
    {
        critical_section::with(|_| {
            let curr = self.v.get();
            self.v.set(curr - v);
            curr
        })
    }
    pub fn fetch_and(&self, v: T) -> T
    where
        T: BitAnd<Output = T>,
    {
        critical_section::with(|_| {
            let curr = self.v.get();
            self.v.set(curr & v);
            curr
        })
    }
    pub fn fetch_nand(&self, v: T) -> T
    where
        T: BitAnd<Output = T> + Not<Output = T>,
    {
        critical_section::with(|_| {
            let curr = self.v.get();
            self.v.set(!(curr & v));
            curr
        })
    }
    pub fn fetch_or(&self, v: T) -> T
    where
        T: BitOr<Output = T>,
    {
        critical_section::with(|_| {
            let curr = self.v.get();
            self.v.set(curr | v);
            curr
        })
    }
    pub fn fetch_xor(&self, v: T) -> T
    where
        T: BitXor<Output = T>,
    {
        critical_section::with(|_| {
            let curr = self.v.get();
            self.v.set(curr ^ v);
            curr
        })
    }
    pub fn fetch_update<F>(&self, mut f: F) -> Result<T, T>
    where
        F: FnMut(T) -> Option<T>,
        T: PartialEq,
    {
        loop {
            let curr = self.load();
            let new = match f(curr) {
                Some(v) => v,
                None => return Err(curr),
            };
            if self.compare_exchange(curr, new).is_ok() {
                return Ok(curr);
            }
        }
    }
    pub fn fetch_max(&self, v: T) -> T
    where
        T: Ord,
    {
        critical_section::with(|_| {
            let curr = self.v.get();
            let max = v.max(curr);
            self.v.set(max);
            curr
        })
    }
    pub fn fetch_min(&self, v: T) -> T
    where
        T: Ord,
    {
        critical_section::with(|_| {
            let curr = self.v.get();
            let min = v.min(curr);
            self.v.set(min);
            curr
        })
    }
}

pub type AtomicBool = Atomic<bool>;
