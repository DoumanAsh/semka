//!Semaphore primitive for Rust
//!
//!## Types
//!
//!Library supplied two types of semaphores, which generally should not be mixed together as it
//!created a mess.
//!
//!#### Binary
//!
//!Binary semaphore is similar to the `Mutex` as it allows singular lock & unlock.
//!
//!This is the approach that should work for most simple use cases.
//!
//!This implementation is available for all targets.
//!
//!```rust
//! use semka::{Sem, BinarySemaphore};
//! let sem = Sem::new().unwrap();
//!
//! let _guard = sem.lock();
//!
//!```
//!
//!#### Counting Semaphore
//!
//!Semaphore state is expressed as atomic integer, which gets decremented(if possible) on `wait`
//!If decrement is not possible (i.e. state is 0) then it awaits for state to get incremented.
//!
//!Meanwhile `signal` increments the counter. If any other tried is locked in `wait`, then it also
//!wakes one of  the locked threads.
//!
//!```rust
//! use semka::{Sem, CountingSemaphore};
//! let sem = Sem::new(0).unwrap();
//!
//! assert!(sem.try_lock().is_none());
//! sem.signal();
//! sem.signal();
//! let _guard = sem.lock();
//! assert!(sem.try_wait());
//! assert!(!sem.try_wait());
//!
//! drop(_guard);
//! assert!(sem.try_wait());
//!```
//!
//!## Platform implementation
//!
//!#### Windows
//!
//!Uses winapi `CreateSemaphoreW`.
//!
//!Implements `CountingSemaphore` interface
//!
//!#### POSIX
//!
//!All POSIX-compliant systems uses `sem_init`
//!But it must be noted that awaiting can be interrupted by the signal
//!
//!POSIX implementation relies on [libc](https://github.com/rust-lang/libc)
//!
//!This includes all `unix` targets and `fuchsia`
//!
//!Implements `CountingSemaphore` interface
//!
//!### Mac
//!
//!Uses `mach` API.
//!
//!Implements `CountingSemaphore` interface
//!
//!### No OS atomic
//!
//!Trivial atomic implementation that performs spin loop
#[cfg(any(all(unix, not(any(target_os = "macos", target_os = "ios"))), target_os = "fuchsia"))]
mod posix;
#[cfg(any(all(unix, not(any(target_os = "macos", target_os = "ios"))), target_os = "fuchsia"))]
pub use posix::Sem;

#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::Sem;

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod mac;
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use mac::Sem;

mod atomic;
pub use atomic::Sem as AtomicSem;

#[cfg(target_os = "unknown")]
pub use atomic::Sem;

///Describes binary Semaphore interface.
///
///It's state can be described as `bool`.
///Any attempt to `wait` would result in locking `self` (i.e. setting state to `true`)
///Which means unless `lock` is dropped, any further `lock` would block calling thread.
///
///Any `CountingSemaphore` can be used as `BinarySemaphore` whenever it's initial count is `1`
pub trait BinarySemaphore: Sized {
    ///Creates new instance, initially unlocked
    fn new() -> Option<Self>;
    ///Locks self, returning guard that unlocks on drop.
    fn lock(&self) -> BinaryLock<'_, Self>;
}

impl<T: CountingSemaphore> BinarySemaphore for T {
    #[inline(always)]
    fn new() -> Option<Self> {
        T::new(1)
    }

    #[inline(always)]
    fn lock(&self) -> BinaryLock<'_, Self> {
        T::wait(self);
        BinaryLock::new(self, T::signal)
    }
}

pub struct BinaryLock<'a, T: BinarySemaphore> {
    sem: &'a T,
    unlock: fn(&'a T),
}

impl<'a, T: BinarySemaphore> BinaryLock<'a, T> {
    #[inline(always)]
    ///Creates new instance providing reference to semaphore and `fn` to perform unlock
    pub fn new(sem: &'a T, unlock: fn(&'a T)) -> Self {
        Self {
            sem,
            unlock,
        }
    }
}

impl<T: BinarySemaphore> Drop for BinaryLock<'_, T> {
    fn drop(&mut self) {
        (self.unlock)(self.sem)
    }
}

///Describes counting Semaphore interface
///
///This primitive provides access to single integer that can be decremented using signal
///and incremented using wait
pub trait CountingSemaphore: Sized {
    ///Creates new instance, returning None on inability to do so.
    ///
    ///`init` is initial value for the semaphore
    fn new(init: u32) -> Option<Self>;

    ///Decrements self, returning immediately if it was signaled.
    ///
    ///Otherwise awaits for `signal`
    fn wait(&self);

    ///Attempts to decrement self, returning whether self was signaled or not.
    ///
    ///Returns `true` if self was signaled
    ///
    ///Returns `false` otherwise
    fn try_wait(&self) -> bool;

    ///Attempts to decrement self within provided time, returning whether self was signaled or not.
    ///
    ///Returns `true` if self was signaled within specified `timeout`
    ///
    ///Returns `false` otherwise
    fn wait_timeout(&self, timeout: core::time::Duration) -> bool;

    ///Increments self, waking any awaiting thread as result
    fn signal(&self);

    ///Gets semaphore's guard, which signal on drop.
    ///
    ///Before guard is created, function will await for semaphore to get decremented.
    fn lock(&self) -> SemaphoreGuard<'_, Self> {
        self.wait();
        SemaphoreGuard {
            sem: self
        }
    }

    ///Attempts to acquire semaphore's guard, which signal on drop.
    ///
    ///If semaphore cannot be decremented at the current moment, returns `None`
    fn try_lock(&self) -> Option<SemaphoreGuard<'_, Self>> {
        match self.try_wait() {
            true => Some(SemaphoreGuard {
                sem: self,
            }),
            false => None,
        }
    }
}

///Semaphore's guard.
///
///Increments(signal) semaphore on drop.
pub struct SemaphoreGuard<'a, T: CountingSemaphore> {
    sem: &'a T,
}

impl<T: CountingSemaphore> Drop for SemaphoreGuard<'_, T> {
    fn drop(&mut self) {
        self.sem.signal();
    }
}
