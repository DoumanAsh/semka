use core::ptr;
use core::ffi::c_void;
use core::sync::atomic::{AtomicPtr, Ordering};

use crate::unlikely;

const WAIT_OBJECT_0: u32 = 0;
const WAIT_TIMEOUT: u32 = 0x00000102;
const INFINITE: u32 = 0xFFFFFFFF;

extern "system" {
    fn CloseHandle(handle: *mut c_void) -> i32;
    fn CreateSemaphoreW(attrs: *mut c_void, initial: i32, max: i32, name: *const u16) -> *mut c_void;
    fn WaitForSingleObject(handle: *mut c_void, timeout_ms: u32) -> u32;
    fn ReleaseSemaphore(handle: *mut c_void, increment: i32, previous_increment: *mut i32) -> i32;
}

///Windows implementation of Semaphore
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
            return unlikely(false);
        }

        let handle = unsafe {
            CreateSemaphoreW(ptr::null_mut(), init as i32, i32::max_value(), ptr::null())
        };

        match self.handle.compare_exchange(ptr::null_mut(), handle, Ordering::SeqCst, Ordering::Acquire) {
            Ok(_) => !handle.is_null(),
            Err(_) => {
                unsafe {
                    CloseHandle(handle);
                }
                unlikely(false)
            }
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
        let result = unsafe {
            WaitForSingleObject(self.handle.load(Ordering::Acquire), INFINITE)
        };

        match result {
            WAIT_OBJECT_0 => (),
            //We cannot really timeout when there is no timeout
            other => panic!("Unexpected result: {}", other),
        }
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
        use core::convert::TryInto;

        let result = unsafe {
            WaitForSingleObject(self.handle.load(Ordering::Acquire), timeout.as_millis().try_into().unwrap_or(u32::max_value()))
        };

        match result {
            WAIT_OBJECT_0 => true,
            WAIT_TIMEOUT => false,
            other => panic!("Unexpected result: {}", other),
        }
    }

    ///Increments self, waking any awaiting thread as result.
    pub fn signal(&self) {
        let res = unsafe {
            ReleaseSemaphore(self.handle.load(Ordering::Acquire), 1, ptr::null_mut())
        };
        debug_assert_ne!(res, 0);
    }


    ///Performs deinitialization.
    ///
    ///Using `Sem` after `close` is undefined behaviour, unless `init` is called
    pub unsafe fn close(&self) {
        let handle = self.handle.swap(ptr::null_mut(), Ordering::AcqRel);
        if !handle.is_null() {
            CloseHandle(handle);
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
