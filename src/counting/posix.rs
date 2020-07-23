use core::cell::UnsafeCell;
#[allow(unused)]
use core::convert::TryFrom;
use core::mem;

use error_code::PosixError;

///POSIX implementation of Semaphore
///
///Note: `wait_timeout` returns false on interrupt by signal
pub struct Sem {
    handle: UnsafeCell<libc::sem_t>,
}

impl super::Semaphore for Sem {
    fn new(init: u32) -> Option<Self> {
        let mut handle = mem::MaybeUninit::uninit();

        let res = unsafe {
            libc::sem_init(handle.as_mut_ptr(), 0, init as libc::c_uint)
        };

        match res {
            0 => Some(Self {
                handle: UnsafeCell::new(unsafe {
                    handle.assume_init()
                })
            }),
            _ => None,
        }
    }

    fn wait(&self) {
        loop {
            let res = unsafe {
                libc::sem_wait(self.handle.get())
            };

            if res == -1 {
                let errno = PosixError::last();
                debug_assert_eq!(errno.raw_code(), libc::EINTR, "Unexpected error");
                continue;
            }

            break
        }
    }

    fn try_wait(&self) -> bool {
        loop {
            let res = unsafe {
                libc::sem_trywait(self.handle.get())
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

    fn wait_timeout(&self, timeout: core::time::Duration) -> bool {
        let timeout = libc::timespec {
            tv_sec: timeout.as_secs() as libc::time_t,
            #[cfg(target_pointer_width = "64")]
            tv_nsec: libc::suseconds_t::from(timeout.subsec_nanos()),
            #[cfg(not(target_pointer_width = "64"))]
            tv_nsec: libc::suseconds_t::try_from(timeout.subsec_nanos()).unwrap_or(libc::suseconds_t::max_value()),
        };

        loop {
            let res = unsafe {
                libc::sem_timedwait(self.handle.get(), &timeout)
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

    fn signal(&self) {
        let res = unsafe {
            libc::sem_post(self.handle.get())
        };
        debug_assert_eq!(res, 0);
    }
}

impl Drop for Sem {
    fn drop(&mut self) {
        unsafe {
            libc::sem_destroy(self.handle.get());
        }
    }
}

unsafe impl Send for Sem {}
unsafe impl Sync for Sem {}
