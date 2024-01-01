use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref STATUS_MUTEX: Mutex<()> = Mutex::new(());
}

pub struct Status {
    n_combinations_checked: usize,
}

impl Status {
    pub fn new() -> Self {
        Self {
            n_combinations_checked: 0,
        }
    }
}

