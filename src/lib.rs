//!Semaphore primitive
//!
//!## Platform implementation
//!
//!#### Windows
//!
//!Uses winapi `CreateSemaphoreW` and fully implements `Semaphore` interface
//!
//!#### POSIX
//!
//!All POSIX-compliant systems uses `sem_init`
//!But it must be noted that `Semaphore::wait_timeout` can be interrupted by the signal
//!
//!POSIX implementation relies on [libc](https://github.com/rust-lang/libc)
//!
//!This includes all `unix` targets and `fuchsia`
//!
//!### Mac
//!
//!Uses `mach` API which doesn't allow to access underlying value, resulting in `Semaphore::post` always returning `false`

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

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

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::Sem;

///Describes Semaphore interface
///
///This primitive provides access to single integer that can be decremented using signal
///and incremented using wait
pub trait Semaphore: Sized {
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

///[Semaphore](trait.Semaphore.html) guard
///
///Increments(signal) semaphore on drop.
pub struct SemaphoreGuard<'a, T: Semaphore> {
    sem: &'a T,
}

impl<T: Semaphore> Drop for SemaphoreGuard<'_, T> {
    fn drop(&mut self) {
        self.sem.signal();
    }
}
