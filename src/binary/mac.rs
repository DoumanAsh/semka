use core::ffi::c_void;
#[allow(unused)]
use core::convert::TryFrom;
use core::mem;

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
const SYNC_POLICY_PREPOST: libc::c_int = 0x04;

extern "C" {
    static mach_task_self_: libc::c_uint;

    //typedef struct semaphore *semaphore_t;
    //Function takes semaphore_t*
    fn semaphore_create(task: libc::c_uint, semaphore: *mut *mut c_void, policy: libc::c_int, value: libc::c_int) -> libc::c_int;
    fn semaphore_signal_all(semaphore: *mut c_void) -> libc::c_int;
    fn semaphore_wait(semaphore: *mut c_void) -> libc::c_int;
    fn semaphore_timedwait(semaphore: *mut c_void, timeout: TimeSpec) -> libc::c_int;
    fn semaphore_destroy(task: libc::c_uint, semaphore: *mut c_void) -> libc::c_int;
}

///MacOS semaphore based on mach API
pub struct Sem {
    handle: *mut c_void,
}

impl super::Semaphore for Sem {
    fn new() -> Option<Self> {
        let mut handle = mem::MaybeUninit::uninit();

        let res = unsafe {
            semaphore_create(mach_task_self_, handle.as_mut_ptr(), SYNC_POLICY_PREPOST, 0)
        };

        match res {
            0 => Some(Self {
                handle: unsafe { handle.assume_init() },
            }),
            _ => None,
        }
    }

    fn wait(&self) {
        let result = unsafe {
            semaphore_wait(self.handle)
        };

        debug_assert_eq!(result, 0, "semaphore_wait() failed");
    }

    #[inline]
    fn try_wait(&self) -> bool {
        self.wait_timeout(core::time::Duration::from_secs(0))
    }

    fn wait_timeout(&self, timeout: core::time::Duration) -> bool {
        let result = unsafe {
            semaphore_timedwait(self.handle, timeout.into())
        };

        debug_assert!(result == 0 || result == KERN_OPERATION_TIMED_OUT, "semaphore_timedwait() failed");
        result == 0
    }

    fn signal(&self) {
        let res = unsafe {
            semaphore_signal_all(self.handle)
        };

        debug_assert_eq!(res, 0);
    }
}

impl Drop for Sem {
    fn drop(&mut self) {
        unsafe {
            semaphore_destroy(mach_task_self_, self.handle);
        }
    }
}

unsafe impl Send for Sem {}
unsafe impl Sync for Sem {}
