use std::sync::{Arc, Mutex};
use std::thread;
use std::vec::Vec;
use crate::ga_core::{Organism, Population};

type ConnectionIdx = usize;
type ConnectionIdxVec = Vec<ConnectionIdx>;
type OrderingIdx = usize;

pub struct GeneticAlgorithm {
    n_organisms_in_population: usize,
    crossover_rate: f64,
    mutation_rate: f64,
    n_connections_in_circuit: usize,
    next_ordering_idx: usize,
    n_unprocessed_orderings: usize,
    mutex: Arc<Mutex<()>>,
    population: Population,
}

impl GeneticAlgorithm {
    pub fn new(n_organisms_in_population: usize, crossover_rate: f64, mutation_rate: f64) -> Self {
        Self {
            n_organisms_in_population,
            crossover_rate,
            mutation_rate,
            n_connections_in_circuit: 0,
            next_ordering_idx: 0,
            n_unprocessed_orderings: 0,
            mutex: Arc::new(Mutex::new(())),
            population: Population::new(n_organisms_in_population, crossover_rate, mutation_rate),
        }
    }

    pub fn reset(&mut self, n_connections_in_circuit: usize) {
        self.n_connections_in_circuit = n_connections_in_circuit;
        self.population.reset(n_connections_in_circuit);
        self.next_ordering_idx = 0;
        self.n_unprocessed_orderings = self.n_organisms_in_population;
    }

    pub fn reserve_ordering(&mut self) -> OrderingIdx {
        if self.n_connections_in_circuit == 0 {
            return usize::MAX;
        }
        let is_new_generation_required = self.next_ordering_idx == self.n_organisms_in_population;
        let is_all_orderings_released = self.n_unprocessed_orderings == 0;
        if is_new_generation_required {
            if is_all_orderings_released {
                self.population.next_generation();
                self.n_unprocessed_orderings = self.n_organisms_in_population;
                self.next_ordering_idx = 0;
            } else {
                return usize::MAX;
            }
        }
        let result = self.next_ordering_idx;
        self.next_ordering_idx += 1;
        result
    }

    pub fn get_ordering(&self, ordering_idx: OrderingIdx) -> ConnectionIdxVec {
        assert_ne!(ordering_idx, usize::MAX); // Must wait and try reserve_ordering() again
        assert_ne!(self.n_connections_in_circuit, 0); // Must call reset() first
        self.population.organism_vec[ordering_idx ].calc_connection_idx_vec()
    }

    pub fn release_ordering(&mut self, ordering_idx: OrderingIdx, n_completed_routes: usize, completed_route_cost: usize) {
        self.population.organism_vec[ordering_idx ].n_completed_routes = n_completed_routes;
        self.population.organism_vec[ordering_idx ].completed_route_cost = completed_route_cost;
        self.n_unprocessed_orderings -= 1;
    }

    pub fn scope_lock(&self) -> std::sync::MutexGuard<()> {
        self.mutex.lock().unwrap()
    }
}
