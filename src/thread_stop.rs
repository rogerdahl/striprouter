use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct ThreadStop {
    mutex: Mutex<()>,
    stopped: Arc<AtomicBool>,
}

impl ThreadStop {
    pub fn new() -> Self {
        Self {
            mutex: Mutex::new(()),
            stopped: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn stop(&self) {
        let _guard = self.mutex.lock().unwrap();
        self.stopped.store(true, Ordering::SeqCst);
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }
}
