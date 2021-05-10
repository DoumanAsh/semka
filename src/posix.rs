use core::mem;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};

use error_code::PosixError;

const UNINIT: u8 = 0;
const INITING: u8 = 0b01;
const INITED: u8 = 0b10;

///POSIX implementation of Semaphore
///
///Note: `wait_timeout` returns false on interrupt by signal
pub struct Sem {
    handle: UnsafeCell<mem::MaybeUninit<libc::sem_t>>,
    state: AtomicU8,
}

impl Sem {
    ///Creates new uninit instance.
    ///
    ///It is UB to use it until `init` is called.
    pub const unsafe fn new_uninit() -> Self {
        Self {
            handle: UnsafeCell::new(mem::MaybeUninit::uninit()),
            state: AtomicU8::new(UNINIT),
        }
    }

    #[must_use]
    ///Initializes semaphore with provided `init` as initial value.
    ///
    ///Returns `true` on success.
    ///
    ///Returns `false` if semaphore is already initialized or initialization failed.
    pub fn init(&self, init: u32) -> bool {
        if let Ok(UNINIT) = self.state.compare_exchange(UNINIT, INITING, Ordering::SeqCst, Ordering::Acquire) {
            let res = unsafe {
                libc::sem_init(self.handle.get() as _, 0, init as _)
            };

            match res {
                0 => {
                    self.state.store(INITED, Ordering::Release);
                    true
                },
                _ => false,
            }
        } else {
            while self.state.load(Ordering::Acquire) != INITED {
                 core::hint::spin_loop();
            }

            false
        }
    }

    ///Creates new instance, initializing it with `init`
    pub fn new(init: u32) -> Option<Self> {
        let result = unsafe {
            Self::new_uninit()
        };

        if result.init(init) {
            Some(result)
        } else {
            None
        }
    }

    ///Decrements self, returning immediately if it was signaled.
    ///
    ///Otherwise awaits for signal.
    pub fn wait(&self) {
        loop {
            let res = unsafe {
                libc::sem_wait(mem::transmute(self.handle.get()))
            };

            if res == -1 {
                let errno = PosixError::last();
                debug_assert_eq!(errno.raw_code(), libc::EINTR, "Unexpected error");
                continue;
            }

            break
        }
    }

    #[inline]
    ///Attempts to decrement self, returning whether self was signaled or not.
    ///
    ///Returns `true` if self was signaled.
    ///
    ///Returns `false` otherwise.
    pub fn try_wait(&self) -> bool {
        loop {
            let res = unsafe {
                libc::sem_trywait(mem::transmute(self.handle.get()))
            };

            if res == -1 {
                let errno = PosixError::last();
                if errno.is_would_block() {
                    break false;
                }

                debug_assert_eq!(errno.raw_code(), libc::EINTR, "Unexpected error");
                continue;
            }

            break true
        }
    }

    ///Attempts to decrement self within provided time, returning whether self was signaled or not.
    ///
    ///Returns `true` if self was signaled within specified timeout
    ///
    ///Returns `false` otherwise
    pub fn wait_timeout(&self, duration: core::time::Duration) -> bool {
        let mut timeout = mem::MaybeUninit::uninit();
        if unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, timeout.as_mut_ptr()) } == -1 {
            panic!("Failed to get current time");
        }

        let mut timeout = unsafe {
            timeout.assume_init()
        };
        timeout.tv_sec = timeout.tv_sec.saturating_add(duration.as_secs() as _);
        timeout.tv_nsec = timeout.tv_nsec.saturating_add(duration.subsec_nanos() as _);
        if timeout.tv_nsec > 999999999 {
            timeout.tv_nsec = 0;
            timeout.tv_sec = timeout.tv_sec.saturating_add(1);
        }

        loop {
            let res = unsafe {
                libc::sem_timedwait(mem::transmute(self.handle.get()), &timeout)
            };

            if res == -1 {
                let errno = PosixError::last();
                if errno.is_would_block() || errno.raw_code() == libc::ETIMEDOUT {
                    break false;
                }

                if errno.raw_code() != libc::EINTR {
                    panic!("Unexpected error: {}", errno);
                }
                continue;
            }

            break true
        }
    }

    ///Increments self, waking any awaiting thread as result.
    pub fn signal(&self) {
        let res = unsafe {
            libc::sem_post(mem::transmute(self.handle.get()))
        };
        debug_assert_eq!(res, 0);
    }
}

impl Drop for Sem {
    fn drop(&mut self) {
        if self.state.load(Ordering::Relaxed) == INITED {
            unsafe {
                libc::sem_destroy(mem::transmute(self.handle.get()));
            }
        }
    }
}

unsafe impl Send for Sem {}
unsafe impl Sync for Sem {}
