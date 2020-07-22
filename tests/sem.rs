use semka::{Sem, Semaphore};

#[test]
fn should_return_when_signaled() {
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
