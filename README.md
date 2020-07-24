# semka

![Rust](https://github.com/DoumanAsh/semka/workflows/Rust/badge.svg?branch=master)
[![Crates.io](https://img.shields.io/crates/v/semka.svg)](https://crates.io/crates/semka)
[![Documentation](https://docs.rs/semka/badge.svg)](https://docs.rs/crate/semka/)

Semaphore primitive for Rust

## Types

Library supplied two types of semaphores, which generally should not be mixed together as it
created a mess.

#### Binary

Binary semaphore is similar to the `Mutex` as it allows singular lock & unlock.

This is the approach that should work for most simple use cases.

This implementation is available for all targets.

```rust
 use semka::{Sem, BinarySemaphore};
 let sem = Sem::new().unwrap();

 let _guard = sem.lock();

```

#### Counting Semaphore

Semaphore state is expressed as atomic integer, which gets decremented(if possible) on `wait`.
If decrement is not possible (i.e. state is 0) then it awaits for state to get incremented.

Meanwhile `signal` increments the counter. If any other tried is locked in `wait`, then it also
wakes one of  the locked threads.

```rust
 use semka::{Sem, CountingSemaphore};
 let sem = Sem::new(0).unwrap();

 assert!(sem.try_lock().is_none());
 sem.signal();
 sem.signal();
 let _guard = sem.lock();
 assert!(sem.try_wait());
 assert!(!sem.try_wait());

 drop(_guard);
 assert!(sem.try_wait());
```

## Platform implementation

#### Windows

Uses winapi `CreateSemaphoreW`.

Implements `CountingSemaphore` interface

#### POSIX

All POSIX-compliant systems uses `sem_init`
But it must be noted that awaiting can be interrupted by the signal

POSIX implementation relies on [libc](https://github.com/rust-lang/libc)

This includes all `unix` targets and `fuchsia`

Implements `CountingSemaphore` interface

### Mac

Uses `mach` API.

Implements `CountingSemaphore` interface

### No OS atomic

Trivial atomic implementation that performs spin loop
