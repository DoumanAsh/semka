use core::sync::atomic::{AtomicBool, Ordering, spin_loop_hint};

///Atomic based Semaphore
///
///Provides only `BinarySemaphore` interface
pub struct Sem {
    counter: AtomicBool,
}

impl Sem {
    #[inline]
    ///Creates new instance with initial value.
    pub const fn new(init: bool) -> Self {
        Self {
            counter: AtomicBool::new(init)
        }
    }

    #[inline]
    fn try_wait(&self) -> bool {
        self.counter.compare_and_swap(true, false, Ordering::SeqCst)
    }

    fn signal(&self) {
        self.counter.store(true, Ordering::SeqCst)
    }
}

impl super::BinarySemaphore for Sem {
    #[inline]
    fn new() -> Option<Self> {
        Some(Self::new(false))
    }

    fn lock(&self) -> super::BinaryLock<'_, Self> {
        while self.try_wait() {
            spin_loop_hint();
        }

        super::BinaryLock::new(self, Self::signal)
    }
}
