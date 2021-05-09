//!Semaphore primitive for Rust
//!
//!## Platform implementation
//!
//!#### Windows
//!
//!Uses winapi `CreateSemaphoreW`.
//!
//!#### POSIX
//!
//!All POSIX-compliant systems uses `sem_init`
//!But it must be noted that awaiting can be interrupted by the signal, although implementation
//!tries its best to handle these cases
//!
//!POSIX implementation relies on [libc](https://github.com/rust-lang/libc)
//!
//!This includes all `unix` targets and `fuchsia`
//!
//!### Mac
//!
//!Uses `mach` API.

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

#[cfg(not(any(windows, unix, target_os = "fuchsia")))]
compile_error!("Semaphore is not available for your target");

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
