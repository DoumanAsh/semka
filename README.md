# semka

![Rust](https://github.com/DoumanAsh/semka/workflows/Rust/badge.svg?branch=master)
[![Crates.io](https://img.shields.io/crates/v/semka.svg)](https://crates.io/crates/semka)
[![Documentation](https://docs.rs/semka/badge.svg)](https://docs.rs/crate/semka/)

Semaphore primitive for Rust

## Platform implementation

#### Windows

Uses winapi `CreateSemaphoreW`.

#### POSIX

All POSIX-compliant systems uses `sem_init`
But it must be noted that awaiting can be interrupted by the signal, although implementation
tries its best to handle these cases

POSIX implementation relies on [libc](https://github.com/rust-lang/libc)

This includes all `unix` targets and `fuchsia`

### Mac

Uses `mach` API.
