use std::array;
use std::ptr;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// Index tagging to solve the ABA problem.
fn pack(index: usize, tag: usize) -> usize {
    (tag << 32) | index
}

/// Index tagging to solve the ABA problem.
fn unpack(value: usize) -> (usize, usize) {
    let index = value & 0xFFFF_FFFF;
    let tag = value >> 32;
    (index, tag)
}

pub struct LockFreeArray<T: Send + Sync, const N: usize> {
    slots: [AtomicPtr<T>; N],
    freelist_head: AtomicUsize, // stores (tag << 32) | index
    next: [AtomicUsize; N],
}

impl<T: Send + Sync, const N: usize> LockFreeArray<T, N> {
    pub fn new() -> Self {
        let slots = array::from_fn(|_| AtomicPtr::new(ptr::null_mut()));
        let next = array::from_fn(|i| AtomicUsize::new(if i + 1 < N { i + 1 } else { N }));

        Self {
            slots,
            freelist_head: AtomicUsize::new(pack(0, 0)),
            next,
        }
    }

    pub fn try_insert(&self, value: T) -> Result<usize, T> {
        let boxed = Box::into_raw(Box::new(value));

        loop {
            let old = self.freelist_head.load(Ordering::Acquire);
            let (head, tag) = unpack(old);

            if head == N {
                let value = unsafe { *Box::from_raw(boxed) };
                return Err(value);
            }

            let next_index = self.next[head].load(Ordering::Relaxed);
            let new = pack(next_index, tag.wrapping_add(1));

            if self
                .freelist_head
                .compare_exchange(old, new, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                self.slots[head].store(boxed, Ordering::Release);
                return Ok(head);
            }
            // Retry if compare_exchange failed
        }
    }

    pub fn take(&self, index: usize) -> Option<T> {
        if index >= N {
            return None;
        }

        let ptr = self.slots[index].swap(ptr::null_mut(), Ordering::AcqRel);
        if ptr.is_null() {
            return None;
        }

        let value = unsafe { *Box::from_raw(ptr) };

        loop {
            let old = self.freelist_head.load(Ordering::Acquire);
            let (head, tag) = unpack(old);

            self.next[index].store(head, Ordering::Relaxed);
            let new = pack(index, tag.wrapping_add(1));

            if self
                .freelist_head
                .compare_exchange(old, new, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        Some(value)
    }
}

impl<T: Send + Sync, const N: usize> Default for LockFreeArray<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn main() {
    let array = LockFreeArray::<i32, 10>::new();

    if let Ok(index) = array.try_insert(42) {
        println!("Inserted 42 at index {}", index);
    } else {
        println!("Failed to insert 42");
    }

    if let Ok(index) = array.try_insert(43) {
        println!("Inserted 43 at index {}", index);
        if let Some(value) = array.take(index) {
            println!("Took value: {}", value);
        } else {
            println!("No value at index 0");
        }
    } else {
        println!("Failed to insert 42");
    }
}
