// Usage for Timer:
//     {
//         let _timer = Timer::new();
//         // Your code here...
//     }
use std::time::Instant;

pub struct Timer {
    start: Instant,
}

impl Timer {
    pub(crate) fn new() -> Timer {
        Timer { start: Instant::now() }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        println!("Time elapsed: {:?}", duration);
    }
}
