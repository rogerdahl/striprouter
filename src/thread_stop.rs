use std::sync::{Mutex, MutexGuard};

pub struct ThreadStop {
    mutex: Mutex<()>,
    lock: Option<MutexGuard<'static, ()>>,
}

impl ThreadStop {
    pub fn new() -> Self {
        Self {
            mutex: Mutex::new(()),
            lock: None,
        }
    }

    pub fn stop(&mut self) {
        // assert!(self.lock.is_none());
        // self.lock = Some(self.mutex.lock().unwrap());
    }

    pub fn is_stopped(&self) -> bool {
        self.lock.is_some()
    }
}

