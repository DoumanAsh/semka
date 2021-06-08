use core::ffi::c_void;
use core::{ptr, mem};
use core::sync::atomic::{AtomicPtr, Ordering};

#[repr(C)]
struct TimeSpec {
    tv_sec: libc::c_uint,
    tv_nsec: libc::c_int,
}

impl Into<TimeSpec> for core::time::Duration {
    fn into(self) -> TimeSpec {
        use core::convert::TryFrom;

        TimeSpec {
            tv_sec: libc::c_uint::try_from(self.as_secs()).unwrap_or(libc::c_uint::max_value()),
            tv_nsec: libc::c_int::try_from(self.subsec_nanos()).unwrap_or(libc::c_int::max_value())
        }
    }
}

const KERN_OPERATION_TIMED_OUT: libc::c_int = 49;
const SYNC_POLICY_FIFO: libc::c_int = 0;

extern "C" {
    static mach_task_self_: libc::c_uint;

    //typedef struct semaphore *semaphore_t;
    //Function takes semaphore_t*
    fn semaphore_create(task: libc::c_uint, semaphore: *mut *mut c_void, policy: libc::c_int, value: libc::c_int) -> libc::c_int;
    fn semaphore_signal(semaphore: *mut c_void) -> libc::c_int;
    fn semaphore_wait(semaphore: *mut c_void) -> libc::c_int;
    fn semaphore_timedwait(semaphore: *mut c_void, timeout: TimeSpec) -> libc::c_int;
    fn semaphore_destroy(task: libc::c_uint, semaphore: *mut c_void) -> libc::c_int;
}

///MacOS semaphore based on mach API
pub struct Sem {
    handle: AtomicPtr<c_void>
}

impl Sem {
    ///Creates new uninit instance.
    ///
    ///It is UB to use it until `init` is called.
    pub const unsafe fn new_uninit() -> Self {
        Self {
            handle: AtomicPtr::new(ptr::null_mut())
        }
    }

    #[must_use]
    ///Initializes semaphore with provided `init` as initial value.
    ///
    ///Returns `true` on success.
    ///
    ///Returns `false` if semaphore is already initialized or initialization failed.
    pub fn init(&self, init: u32) -> bool {
        if !self.handle.load(Ordering::Acquire).is_null() {
            return false;
        }

        let mut handle = mem::MaybeUninit::uninit();

        let res = unsafe {
            semaphore_create(mach_task_self_, handle.as_mut_ptr(), SYNC_POLICY_FIFO, init as libc::c_int)
        };

        match res {
            0 => unsafe {
                let handle = handle.assume_init();
                match self.handle.compare_exchange(ptr::null_mut(), handle, Ordering::SeqCst, Ordering::Acquire) {
                    Ok(_) => true,
                    Err(_) => {
                        semaphore_destroy(mach_task_self_, handle);
                        false
                    }
                }
            },
            _ => false,
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
        let result = unsafe {
            semaphore_wait(self.handle.load(Ordering::Acquire))
        };

        debug_assert_eq!(result, 0, "semaphore_wait() failed");
    }

    #[inline]
    ///Attempts to decrement self, returning whether self was signaled or not.
    ///
    ///Returns `true` if self was signaled.
    ///
    ///Returns `false` otherwise.
    pub fn try_wait(&self) -> bool {
        self.wait_timeout(core::time::Duration::from_secs(0))
    }

    ///Attempts to decrement self within provided time, returning whether self was signaled or not.
    ///
    ///Returns `true` if self was signaled within specified timeout
    ///
    ///Returns `false` otherwise
    pub fn wait_timeout(&self, timeout: core::time::Duration) -> bool {
        let result = unsafe {
            semaphore_timedwait(self.handle.load(Ordering::Acquire), timeout.into())
        };

        debug_assert!(result == 0 || result == KERN_OPERATION_TIMED_OUT, "semaphore_timedwait() failed");
        result == 0
    }

    ///Increments self, waking any awaiting thread as result.
    pub fn signal(&self) {
        let res = unsafe {
            semaphore_signal(self.handle.load(Ordering::Acquire))
        };

        debug_assert_eq!(res, 0, "semaphore_signal() failed");
    }

    ///Performs deinitialization.
    ///
    ///Using `Sem` after `close` is undefined behaviour, unless `init` is called
    pub unsafe fn close(&self) {
        let handle = self.handle.swap(ptr::null_mut(), Ordering::AcqRel);
        if !handle.is_null() {
            semaphore_destroy(mach_task_self_, handle);
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
