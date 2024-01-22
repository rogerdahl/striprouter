use crate::ga_interface::GeneticAlgorithm;
use crate::layout::Layout;
// use crate::thread_stop::ThreadStop;
use crate::via::Via;
use crate::{nets, router, settings};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

pub(crate) struct RouterThread {
    input_layout: Arc<Mutex<Layout>>,
    current_layout: Arc<Mutex<Layout>>,
    best_layout: Arc<Mutex<Layout>>,
    router_stop_signal: Arc<(Mutex<bool>, Condvar)>,
    genetic_algorithm: Arc<Mutex<GeneticAlgorithm>>,
    counter: Arc<AtomicUsize>,
    limit_routes: Arc<AtomicUsize>,
    thread_idx: usize,
}

impl RouterThread {
    pub fn new(
        // router_stop_signal: Arc<Mutex<ThreadStop>>,
        input_layout: Arc<Mutex<Layout>>,
        current_layout: Arc<Mutex<Layout>>,
        best_layout: Arc<Mutex<Layout>>,
        router_stop_signal: Arc<(Mutex<bool>, Condvar)>,
        genetic_algorithm: Arc<Mutex<GeneticAlgorithm>>,
        counter: Arc<AtomicUsize>,
        limit_routes: Arc<AtomicUsize>,
        thread_idx: usize,
    ) -> Self {
        Self {
            input_layout,
            current_layout,
            best_layout,
            router_stop_signal,
            genetic_algorithm,
            counter,
            limit_routes,
            thread_idx,
        }
    }
    pub fn run(&mut self) {
        // Yes, the unwrap() method can fail if the lock() method returns a Result
        // that is an Err. This would occur if the mutex is poisoned. A mutex is
        // poisoned whenever a thread panics while holding the lock. When a mutex is
        // poisoned, any subsequent calls to lock() will return an Err.

        // println!("run() thread_idx={}", self.thread_idx);

        // println!("waittttt thread_idx={}", self.thread_idx);
        //
        // while !*stop {
        let (lock, cvar) = &*self.router_stop_signal;
        loop {
            // We check the stop signal in a separate scope to avoid holding the lock
            // permanently in the loop.
            {
                let mut stop = lock.lock().unwrap();
                // println!("run() thread_idx={} stop={}", self.thread_idx, *stop);
                if *stop {
                    break;
                }
            }
            // println!("while thread_idx={}", self.thread_idx);
            // while !self.router_stop_signal.lock().unwrap().is_stopped() {
            let mut ordering_idx;
            loop {
                ordering_idx = self.genetic_algorithm.lock().unwrap().reserve_ordering();
                if ordering_idx != usize::MAX {
                    break;
                } else {
                    // println!("thread_idx={} waiting", self.thread_idx);
                    thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            // let ordering = self.genetic_algorithm.lock().unwrap().get_ordering(ordering_idx);
            // println!("ordering={:?}", ordering);


            let ordering = (0..self.input_layout.lock().unwrap().circuit.connection_vec.len()).collect::<Vec<usize>>();
            // println!("ordering={:?}", ordering);

            // {
            //     let mut x = self.input_layout.lock().unwrap();
            //     println!("x.layout_info_vec.len() = {}", x.layout_info_vec.len());
            //     println!("x.route_status_vec.len() = {}", x.route_status_vec.len());
            //     println!("x.set_idx_vec.len() = {}", x.set_idx_vec.len());
            //     println!("x.strip_cut_vec.len() = {}", x.strip_cut_vec.len());
            //     println!("x.via_set_vec.len() = {}", x.via_set_vec.len());
            //     println!("x.route_vec.len() = {}", x.route_vec.len());
            //     println!("x.n_completed_routes = {}", x.n_completed_routes);
            //     println!("x.n_failed_routes = {}", x.n_failed_routes);
            //     println!("x.cost = {}", x.cost);
            // }

            let mut thread_layout = self.input_layout.lock().unwrap().thread_safe_copy();

            let mut router = router::Router::new(thread_layout.board);
            let mut nets = nets::Nets::new(thread_layout.board);
            let settings = settings::Settings::new();

            let connection_idx_vec = ordering;


            router.route(
                thread_layout.board,
                &mut thread_layout,
                &mut nets,
                connection_idx_vec,
                &mut self.limit_routes,
            );

            // thread_layout.via_set_vec = nets.via_set_vec;
            // thread_layout.set_idx_vec = nets.set_idx_vec;

            self.genetic_algorithm.lock().unwrap().release_ordering(
                ordering_idx,
                thread_layout.n_completed_routes,
                thread_layout.cost,
            );

            /////////////////////////

            // let mut input_layout_guard = self.input_layout.lock().unwrap();

            // {
                let mut best_layout_guard = self.best_layout.lock().unwrap();
            //
            //     let has_more_completed_routes = thread_layout.n_completed_routes > best_layout_guard.n_completed_routes;
            //     let has_equal_routes_and_better_score = thread_layout.n_completed_routes == best_layout_guard.n_completed_routes && thread_layout.cost < best_layout_guard.cost;
            //
            //     // Assuming `is_based_on` is a method in the `Layout` struct that takes a reference to another `Layout`
            //     // let is_based_on_other_layout = !best_layout_guard.is_based_on(&thread_layout);
            //     let is_based_on_other_layout = false;
            //
            //     if has_more_completed_routes || has_equal_routes_and_better_score || is_based_on_other_layout {
            //         *best_layout_guard = thread_layout.clone();
            //     }
            // }

            println!("completed_routes={}", thread_layout.n_completed_routes);

            // let mut best_layout_guard = self.best_layout.lock().unwrap();
            *best_layout_guard = thread_layout.clone();


            ///////////////////////

            // println!("thread_idx={}", self.thread_idx);
            self.counter.fetch_add(1, Ordering::SeqCst);
            // println!("x={}", x);
            // thread::sleep(std::time::Duration::from_millis(1000));
        }
    }
}

//
//     pub fn router_thread(&mut self) {
//         // Yes, the unwrap() method can fail if the lock() method returns a Result
//         // that is an Err. This would occur if the mutex is poisoned. A mutex is
//         // poisoned whenever a thread panics while holding the lock. When a mutex is
//         // poisoned, any subsequent calls to lock() will return an Err.
//         while !self.router_stop_signal.lock().unwrap().is_stopped() {
//             //     let mut thread_layout;
//             //     {
//             //         let _lock = input_layout.scope_lock();
//             //         if !input_layout.is_ready_for_routing || input_layout.settings.pause {
//             //             std::thread::sleep(Duration::from_millis(10));
//             //             continue;
//             //         }
//             //         thread_layout = input_layout.clone();
//             //     }
//             //     let mut ordering_idx = -1;
//             //     let mut connection_idx_vec = Vec::new();
//             //     if use_random_search {
//             //         for i in 0..thread_layout.circuit.connection_vec.len() {
//             //             connection_idx_vec.push(i);
//             //         }
//             //         connection_idx_vec.shuffle(&mut rand::thread_rng());
//             //     } else {
//             //         {
//             //             let _lock = genetic_algorithm.scope_lock();
//             //             ordering_idx = genetic_algorithm.reserve_ordering();
//             //             if ordering_idx != -1 {
//             //                 connection_idx_vec = genetic_algorithm.get_ordering(ordering_idx);
//             //             }
//             //         }
//             //         if ordering_idx == -1 {
//             //             std::thread::sleep(Duration::from_millis(10));
//             //             continue;
//             //         }
//             //     }
//             //     {
//             //         let router = Router::new(
//             //             thread_layout.clone(), connection_idx_vec.clone(), router_stop_signal.clone(),
//             //             input_layout.clone(), current_layout.clone(), max_render_delay
//             //         );
//             //         let is_aborted = router.route();
//             //         if is_aborted || !thread_layout.is_based_on(&input_layout) {
//             //             continue;
//             //         }
//             //     }
//             //     {
//             //         let _lock = status_mutex.lock().unwrap();
//             //         status.n_combinations_checked += 1;
//             //     }
//             //     if !use_random_search {
//             //         let _lock = genetic_algorithm.scope_lock();
//             //         genetic_algorithm.release_ordering(
//             //             ordering_idx, thread_layout.n_completed_routes, thread_layout.cost
//             //         );
//             //     }
//             //     {
//             //         let _lock = current_layout.scope_lock();
//             //         *current_layout = thread_layout.clone();
//             //     }
//             //     {
//             //         let _input_lock = input_layout.scope_lock();
//             //         let _best_lock = best_layout.scope_lock();
//             //         let has_more_completed_routes =
//             //             thread_layout.n_completed_routes > best_layout.n_completed_routes;
//             //         let has_equal_routes_and_better_score =
//             //             thread_layout.n_completed_routes == best_layout.n_completed_routes
//             //             && thread_layout.cost < best_layout.cost;
//             //         let is_based_on_other_layout = !best_layout.is_based_on(&thread_layout);
//             //         if has_more_completed_routes || has_equal_routes_and_better_score
//             //             || is_based_on_other_layout {
//             //             *best_layout = thread_layout.clone();
//             //         }
//             //     }
//             //     if checkpoint_at_num_checks != -1 {
//             //         let _lock = status_mutex.lock().unwrap();
//             //         if status.n_combinations_checked % checkpoint_at_num_checks == 0 {
//             //             print_stats();
//             //         }
//             //     }
//             //     if exit_on_first_complete {
//             //         let _best_lock = best_layout.scope_lock();
//             //         if best_layout.n_failed_routes == 0 {
//             //             exit_app();
//             //         }
//             //     }
//             //     if exit_after_num_checks != -1 {
//             //         let _lock = status_mutex.lock().unwrap();
//             //         if status.n_combinations_checked == exit_after_num_checks {
//             //             exit_app();
//             //         }
//             //     }
//         }
//     }
// }
