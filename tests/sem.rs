#[test]
fn should_return_when_signaled_counting() {
    use semka::counting::{Sem, Semaphore};

    let sem = Sem::new(0).unwrap();

    assert!(sem.try_lock().is_none());
    sem.signal();
    sem.signal();
    let _guard = sem.lock();
    assert!(sem.try_wait());
    assert!(!sem.try_wait());

    drop(_guard);
    assert!(sem.try_wait());
}

#[test]
fn should_return_when_signaled_binary() {
    use semka::binary::{Sem, Semaphore};

    let sem = Sem::new().unwrap();

    assert!(sem.try_lock().is_none());
    sem.signal();
    sem.signal();
    let _guard = sem.lock();
    assert!(!sem.try_wait());

    drop(_guard);
    assert!(sem.try_wait());
}

#[test]
fn should_return_when_signaled_atomic_binary() {
    use semka::binary::{AtomicSem, Semaphore};

    let sem = AtomicSem::new(false);

    assert!(sem.try_lock().is_none());
    sem.signal();
    sem.signal();
    let _guard = sem.lock();
    assert!(!sem.try_wait());

    drop(_guard);
    assert!(sem.try_wait());
}
