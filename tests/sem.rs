use semka::Sem;

#[test]
fn should_fail_init_twice() {
    let sem = unsafe {
        Sem::new_uninit()
    };

    assert!(sem.init(0));
    assert!(!sem.init(0));

    assert!(!sem.try_wait());
    sem.signal();
    sem.signal();
    assert!(sem.try_wait());
    assert!(sem.try_wait());

    assert!(!sem.try_wait());
}

#[test]
fn should_return_when_signaled_counting() {
    let sem = Sem::new(0).unwrap();

    assert!(!sem.try_wait());
    sem.signal();
    sem.signal();
    assert!(sem.try_wait());
    assert!(sem.try_wait());

    assert!(!sem.try_wait());
}
