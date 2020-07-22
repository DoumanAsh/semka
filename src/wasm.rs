///WASM compatible Semaphore
///
///It relies on `Atomics`
pub struct Sem {
    value: js_sys::Int32Array,
}

impl Sem {
    #[inline]
    fn is_zero(&self) -> bool {
        match js_sys::Atomics::compare_exchange(&self.value, 0, 0, 0) {
            Ok(val) => val == 0,
            Err(_) => unreachable!(),
        }
    }
}

impl super::Semaphore for Sem {
    fn new(init: u32) -> Option<Self> {
        let value = js_sys::Int32Array::new(&js_sys::SharedArrayBuffer::new(1));
        value.set_index(0, init as i32);
        Some(Self {
            value
        })
    }

    fn wait(&self) {
        if self.is_zero() {
            if let Err(error) = js_sys::Atomics::wait(&self.value, 0, 1) {
                panic!("Failed to await: {:?}", error);
            }
        }

        let _ = js_sys::Atomics::sub(&self.value, 0, 1);
    }

    fn try_wait(&self) -> bool {
        match self.is_zero() {
            true => false,
            false => {
                let _ = js_sys::Atomics::sub(&self.value, 0, 1);
                true
            }
        }
    }

    fn wait_timeout(&self, timeout: core::time::Duration) -> bool {
        if self.is_zero() {
            let res = match js_sys::Atomics::wait_with_timeout(&self.value, 0, 1, timeout.as_millis() as _) {
                Ok(res) => res,
                Err(error) => panic!("Failed to await: {:?}", error),
            };

            if res == "timed-out" {
                return false
            }
        }

        let _ = js_sys::Atomics::sub(&self.value, 0, 1);
        return true;
    }

    fn signal(&self) {
        let _ = js_sys::Atomics::add(&self.value, 0, 1);
        let _ = js_sys::Atomics::notify_with_count(&self.value, 0, 1);
    }
}

impl Drop for Sem {
    fn drop(&mut self) {
        let _ = js_sys::Atomics::notify(&self.value, 0);
    }
}
