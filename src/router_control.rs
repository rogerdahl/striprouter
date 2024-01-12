use crate::ga_interface::GeneticAlgorithm;
use crate::layout::Layout;
// use crate::thread_stop::ThreadStop;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Condvar, Mutex};
// Condvar - block and wait:  https://doc.rust-lang.org/std/sync/struct.Condvar.html
use crate::router_thread::RouterThread;
use std::thread;

pub(crate) struct RouterControl {
    input_layout: Arc<Mutex<Layout>>,
    current_layout: Arc<Mutex<Layout>>,
    best_layout: Arc<Mutex<Layout>>,

    // router_stop_signal: Arc<Mutex<ThreadStop>>,
    router_stop_signal: Arc<(Mutex<bool>, Condvar)>,

    // This is where the single instance of the GA is stored. It is used by all
    // router threads.
    genetic_algorithm: Arc<Mutex<GeneticAlgorithm>>,

    router_thread_vec: Vec<RouterThread>,

    counter: Arc<AtomicUsize>,
}

impl RouterControl {
    pub fn new(
        // router_stop_signal: Arc<Mutex<ThreadStop>>,
        input_layout: Arc<Mutex<Layout>>,
        current_layout: Arc<Mutex<Layout>>,
        best_layout: Arc<Mutex<Layout>>,
        counter: Arc<AtomicUsize>,
        // current_layout: Arc<Mutex<Layout>>,
    ) -> Self {
        Self {
            input_layout,
            current_layout,
            best_layout,
            router_stop_signal: Arc::new((Mutex::new(false), Condvar::new())),
            genetic_algorithm: Arc::new(Mutex::new(GeneticAlgorithm::new(1000, 0.7, 0.01))),
            router_thread_vec: Vec::new(),
            counter,
        }
    }

    pub fn start(&mut self) {
        self.genetic_algorithm
            .lock()
            .unwrap()
            .reset(self.input_layout.lock().unwrap().circuit.connection_vec.len());

        // for i in 0..1 {
        for i in 0..num_cpus::get() {
            println!("Starting router thread i: {}", i);
            let router_thread = Arc::new(Mutex::new(RouterThread::new(
                // Arc::clone() is a method on the Arc type that returns a new Arc
                // that points to the same data as the original Arc. It does not clone
                // the underlying data; it only increments the reference count.
                Arc::clone(&self.input_layout),
                Arc::clone(&self.current_layout),
                Arc::clone(&self.best_layout),
                Arc::clone(&self.router_stop_signal),
                Arc::clone(&self.genetic_algorithm),
                Arc::clone(&self.counter),
                i,
            )));
            // let router_thread_clone = Arc::clone(&router_thread);
            thread::spawn(move || {
                let mut router_thread = router_thread.lock().unwrap();
                router_thread.run();
            });
            // &self.router_thread_vec.push(router_thread.lock().unwrap().clone());
            // println!("router_thread_vec.len(): {}", self.router_thread_vec.len());
            // println!("router_thread_vec: {:?}", self.router_thread_vec);
        }
    }

    // pub fn start(&self, shared_self: Arc<Mutex<Self>>) {
    //     let thread_self = Arc::clone(&shared_self);
    //     thread::spawn(move || {
    //         let mut router_thread = thread_self.lock().unwrap();
    //         router_thread.run();
    //     });
    // }
}
