use crate::thread_stop::ThreadStop;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use crate::ga_interface::GeneticAlgorithm;
use crate::layout::Layout;

pub(crate) struct RouterThread {
    thread_stop_router: Arc<ThreadStop>,
    c: i32,
    // thread_vec: Vec<Arc<Mutex<RouterThread>>>,
    thread_vec: Vec<JoinHandle<()>>,
    genetic_algorithm: Arc<Mutex<GeneticAlgorithm>>,
}

impl RouterThread {
    pub fn new(thread_stop_router: Arc<ThreadStop>) -> Self {
        Self {
            thread_stop_router,
            c: 0,
            thread_vec: Vec::new(),
            genetic_algorithm: Arc::new(Mutex::new(GeneticAlgorithm::new(1000, 0.7, 0.01))),
        }
    }

    pub fn start(&mut self, layout: Arc<Mutex<Layout>>) {
        // self.genetic_algorithm.lock().unwrap().reset(layout.lock().unwrap().circuit.connection_vec.len());

        // let mut handles = Vec::new();

        for i in 0..num_cpus::get() {
            println!("i: {}", i);
            // let router_thread_clone = Arc::clone(&router_thread);
            let handle = thread::spawn(move || {
                // let router_thread = Arc::new(Mutex::new(RouterThread::new(Arc::clone(
                //     &self.thread_stop_router,
                // ))));

                let mut router_thread = RouterThread::new(self.thread_stop_router);
                router_thread.run();

                // let mut router_thread = router_thread.lock().unwrap();
                // router_thread.run();
            });

            // handles.push(handle);
            &self.thread_vec.push(handle);
        }
    }

    // pub fn start(&self, shared_self: Arc<Mutex<Self>>) {
    //     let thread_self = Arc::clone(&shared_self);
    //     thread::spawn(move || {
    //         let mut router_thread = thread_self.lock().unwrap();
    //         router_thread.run();
    //     });
    // }

    pub fn run(&mut self) {
        while !self.thread_stop_router.is_stopped() {
            self.c += 1;
            println!("c: {}", self.c);

            // let ordering_idx = self.genetic_algorithm.reserve_ordering();
            // let ordering = g.get_ordering(ordering_idx);
            // println!("ordering={:?}", ordering);
            // let mut cost = 0;
            // for i in 0..ordering.len() {
            //     cost += ordering[i] * (i + 1);
            // }
            // g.release_ordering(ordering_idx, 10, cost);

            thread::sleep(std::time::Duration::from_millis(1000));
        }
    }

    pub fn router_thread(&mut self) {
        while !self.thread_stop_router.is_stopped() {
            //     let mut thread_layout;
            //     {
            //         let _lock = input_layout.scope_lock();
            //         if !input_layout.is_ready_for_routing || input_layout.settings.pause {
            //             std::thread::sleep(Duration::from_millis(10));
            //             continue;
            //         }
            //         thread_layout = input_layout.clone();
            //     }
            //     let mut ordering_idx = -1;
            //     let mut connection_idx_vec = Vec::new();
            //     if use_random_search {
            //         for i in 0..thread_layout.circuit.connection_vec.len() {
            //             connection_idx_vec.push(i);
            //         }
            //         connection_idx_vec.shuffle(&mut rand::thread_rng());
            //     } else {
            //         {
            //             let _lock = genetic_algorithm.scope_lock();
            //             ordering_idx = genetic_algorithm.reserve_ordering();
            //             if ordering_idx != -1 {
            //                 connection_idx_vec = genetic_algorithm.get_ordering(ordering_idx);
            //             }
            //         }
            //         if ordering_idx == -1 {
            //             std::thread::sleep(Duration::from_millis(10));
            //             continue;
            //         }
            //     }
            //     {
            //         let router = Router::new(
            //             thread_layout.clone(), connection_idx_vec.clone(), thread_stop_router.clone(),
            //             input_layout.clone(), current_layout.clone(), max_render_delay
            //         );
            //         let is_aborted = router.route();
            //         if is_aborted || !thread_layout.is_based_on(&input_layout) {
            //             continue;
            //         }
            //     }
            //     {
            //         let _lock = status_mutex.lock().unwrap();
            //         status.n_combinations_checked += 1;
            //     }
            //     if !use_random_search {
            //         let _lock = genetic_algorithm.scope_lock();
            //         genetic_algorithm.release_ordering(
            //             ordering_idx, thread_layout.n_completed_routes, thread_layout.cost
            //         );
            //     }
            //     {
            //         let _lock = current_layout.scope_lock();
            //         *current_layout = thread_layout.clone();
            //     }
            //     {
            //         let _input_lock = input_layout.scope_lock();
            //         let _best_lock = best_layout.scope_lock();
            //         let has_more_completed_routes =
            //             thread_layout.n_completed_routes > best_layout.n_completed_routes;
            //         let has_equal_routes_and_better_score =
            //             thread_layout.n_completed_routes == best_layout.n_completed_routes
            //             && thread_layout.cost < best_layout.cost;
            //         let is_based_on_other_layout = !best_layout.is_based_on(&thread_layout);
            //         if has_more_completed_routes || has_equal_routes_and_better_score
            //             || is_based_on_other_layout {
            //             *best_layout = thread_layout.clone();
            //         }
            //     }
            //     if checkpoint_at_num_checks != -1 {
            //         let _lock = status_mutex.lock().unwrap();
            //         if status.n_combinations_checked % checkpoint_at_num_checks == 0 {
            //             print_stats();
            //         }
            //     }
            //     if exit_on_first_complete {
            //         let _best_lock = best_layout.scope_lock();
            //         if best_layout.n_failed_routes == 0 {
            //             exit_app();
            //         }
            //     }
            //     if exit_after_num_checks != -1 {
            //         let _lock = status_mutex.lock().unwrap();
            //         if status.n_combinations_checked == exit_after_num_checks {
            //             exit_app();
            //         }
            //     }
        }
    }
}
