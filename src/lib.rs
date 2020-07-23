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
//!

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

pub mod binary;
pub mod counting;
