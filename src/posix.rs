use core::mem;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};

use error_code::ErrorCode;

use crate::unlikely;

const UNINIT: u8 = 0;
const INITING: u8 = 0b01;
const INITED: u8 = 0b10;

///POSIX implementation of Semaphore
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

    #[inline(always)]
    ///Returns whether semaphore is successfully initialized
    pub fn is_init(&self) -> bool {
        self.state.load(Ordering::Acquire) == INITED
    }

    #[cold]
    #[inline(never)]
    fn await_init(&self) {
        //Wait for initialization to finish
        while self.state.load(Ordering::Acquire) == INITING {
            core::hint::spin_loop();
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

            let res = match res {
                0 => {
                    self.state.store(INITED, Ordering::Release);
                    true
                },
                _ => {
                    //TODO: assert against?
                    self.state.store(UNINIT, Ordering::Release);
                    false
                },
            };

            unlikely(res)
        } else {
            //Similarly to `Once` we give priority to already-init path
            //although we do need to make sure it is finished
            if self.state.load(Ordering::Acquire) != INITED {
                self.await_init();
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
            unlikely(None)
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
                let errno = ErrorCode::last_posix();
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
                let errno = ErrorCode::last_posix().raw_code();
                if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
                    break false;
                }

                debug_assert_eq!(errno, libc::EINTR, "Unexpected error");
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
                let errno = ErrorCode::last_posix();
                if errno.raw_code() == libc::EAGAIN || errno.raw_code() == libc::EWOULDBLOCK || errno.raw_code() == libc::ETIMEDOUT {
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


    ///Performs deinitialization.
    ///
    ///Using `Sem` after `close` is undefined behaviour, unless `init` is called
    pub unsafe fn close(&self) {
        let handle = self.handle.get();
        if let Ok(INITED) = self.state.compare_exchange(INITED, UNINIT, Ordering::SeqCst, Ordering::Acquire) {
            libc::sem_destroy(mem::transmute(handle));
        }
    }
}

impl Drop for Sem {
    fn drop(&mut self) {
        unsafe {
            self.close();
        }
    }
}

unsafe impl Send for Sem {}
unsafe impl Sync for Sem {}
