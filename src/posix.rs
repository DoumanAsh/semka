use core::mem;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};
use core::convert::TryFrom;

use error_code::PosixError;

///POSIX implementation of Semaphore
///
///Note: `wait_timeout` returns false on interrupt by signal
pub struct Sem {
    handle: UnsafeCell<mem::MaybeUninit<libc::sem_t>>,
    is_init: AtomicBool,
}

impl Sem {
    ///Creates new uninit instance.
    ///
    ///It is UB to use it until `init` is called.
    pub const unsafe fn new_uninit() -> Self {
        Self {
            handle: UnsafeCell::new(mem::MaybeUninit::uninit()),
            is_init: AtomicBool::new(false),
        }
    }

    #[must_use]
    ///Initializes semaphore with provided `init` as initial value.
    ///
    ///Returns `true` on success.
    ///
    ///Returns `false` if semaphore is already initialized or initialization failed.
    pub fn init(&self, init: u32) -> bool {
        if let Ok(false) = self.is_init.compare_exchange(false, true, Ordering::SeqCst, Ordering::Acquire) {
            let mut handle = mem::MaybeUninit::uninit();

            let res = unsafe {
                libc::sem_init(handle.as_mut_ptr(), 0, init as _)
            };

            match res {
                0 => unsafe {
                    *self.handle.get() = handle;
                    true
                },
                _ => false,
            }
        } else {
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
    pub fn wait_timeout(&self, timeout: core::time::Duration) -> bool {
        let timeout = libc::timespec {
            tv_sec: timeout.as_secs() as libc::time_t,
            tv_nsec: libc::suseconds_t::try_from(timeout.subsec_nanos()).unwrap_or(libc::suseconds_t::max_value()),
        };

        loop {
            let res = unsafe {
                libc::sem_timedwait(mem::transmute(self.handle.get()), &timeout)
            };

            if res == -1 {
                let errno = PosixError::last();
                if errno.is_would_block() || errno.raw_code() == libc::ETIMEDOUT {
                    break false;
                }

                debug_assert_eq!(errno.raw_code(), libc::EINTR, "Unexpected error");
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
        unsafe {
            libc::sem_destroy(mem::transmute(self.handle.get()));
        }
    }
}

unsafe impl Send for Sem {}
unsafe impl Sync for Sem {}
