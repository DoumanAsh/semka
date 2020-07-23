use semka::{Sem};

#[test]
fn should_return_when_signaled_counting() {
    use semka::{CountingSemaphore};
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
    use semka::{BinarySemaphore};

    static mut SEM: Option<Sem> = None;

    unsafe {
        SEM = Sem::new();
        assert!(SEM.is_some());
    }

    let thread = std::thread::spawn(|| {
        let _lock = unsafe {
            SEM.as_ref().unwrap().lock()
        };
        std::thread::sleep(core::time::Duration::from_secs(2));
    });

    std::thread::sleep(core::time::Duration::from_secs(1));
    let _lock = unsafe {
        SEM.as_ref().unwrap().lock()
    };

    thread.join().expect("To finish");
}

#[test]
fn should_return_when_signaled_atomic_binary() {
    use semka::{AtomicSem, BinarySemaphore};

    static SEM: AtomicSem = AtomicSem::new(false);

    let thread = std::thread::spawn(|| {
        let _lock = SEM.lock();
        std::thread::sleep(core::time::Duration::from_secs(2));
    });

    std::thread::sleep(core::time::Duration::from_secs(1));
    let _lock = SEM.lock();

    thread.join().expect("To finish");
}
