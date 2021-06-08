use semka::Sem;
use std::time;

#[test]
fn should_init_after_close() {
    let sem = Sem::new(0).unwrap();

    assert!(!sem.init(0));

    unsafe {
        sem.close();
        sem.close();
    }

    assert!(sem.init(0));
    assert!(!sem.init(0));

    assert!(!sem.try_wait());
    sem.signal();
    sem.signal();
    assert!(sem.try_wait());
    assert!(sem.try_wait());

    assert!(!sem.try_wait());

    unsafe {
        sem.close();
        sem.close();
    }
}

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

#[test]
fn should_timeout_on_wait() {
    let sem = Sem::new(0).unwrap();
    assert!(!sem.try_wait());

    let before = time::Instant::now();
    assert!(!sem.wait_timeout(time::Duration::from_millis(2500)));
    let after = time::Instant::now();

    let duration = after.duration_since(before);
    println!("duration={:?}", duration);
    assert!(duration.as_millis() > 2000 && duration.as_millis() < 3000);
}
