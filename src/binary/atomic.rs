use core::sync::atomic::{AtomicBool, Ordering, spin_loop_hint};

///Atomic based Semaphore
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
}

impl super::Semaphore for Sem {
    #[inline]
    fn new() -> Option<Self> {
        Some(Self::new(false))
    }

    fn wait(&self) {
        while self.try_wait() {
            spin_loop_hint();
        }
    }

    #[inline]
    fn try_wait(&self) -> bool {
        self.counter.compare_and_swap(true, false, Ordering::SeqCst)
    }

    fn wait_timeout(&self, _: core::time::Duration) -> bool {
        unimplemented!();
    }

    fn signal(&self) {
        self.counter.store(true, Ordering::SeqCst)
    }
}
