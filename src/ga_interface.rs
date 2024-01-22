use crate::ga_core::{Organism, Population};
use std::sync::{Arc, Mutex};
use std::thread;
use std::vec::Vec;

// Genetic algorithms work in the world of organisms, genes and populations, and
// are built around a batch process, where the fitness is first determined for
// all organisms in the current population and the result is used to create the
// next generation.
//
// In contrast, the rest of this app works in the world of circuits, routes and
// connections. The app just wants to receive an infinite series of (hopefully)
// gradually improving route orderings to be checked for a circuit.
//
// This class, GeneticAlgorithm, wraps the actual GA stuff and translates
// between the two worlds by providing a black box from which an infinite series
// of orderings can be retrieved. The result is returned as a simple vector of
// connection indexes.
//
// As all operations on the underlying GA objects are performed through this
// class, the locking required by this class protects the rest of the GA system.
//
// The app uses multiple threads to check multiple orderings at the same time.
// At the same time, each thread needs to be able to pass back the cost for the
// layout that resulted from the ordering it received since it becomes the
// fitness for the corresponding organism. To support this, the class works as
// follows:
//
// - The client creates a single global instance of GeneticAlgorithm.
// - The client calls reset() whenever the layout changes, which sets up an
// initial population with randomized genes.
// - TODO: When the layout changes, maybe it's better to just keep going on the
// existing population.
// - The object keeps track of how many organisms there are in the
// population and how many organisms have received fitness scores.
// - The object has a single lock and before a thread interacts with the
// object, it must obtain the lock.
// - A thread first retrieves the index of an ordering by calling
// reserveOrdering().
// - The thread then retrieves the actual ordering by calling getOrdering(),
// passing the index.
// - The thread routes the ordering.
// - The thread releases the ordering and provides the cost of the
// resulting layout by calling releaseOrdering(), passing the index and the
// cost.
// - When a fitness score has been received for each organism in the population,
// the creation of a new generation in triggered by the next call to
// reserveOrdering(). The index for the first ordering in the new generation
// (index 0) is then returned.
// - Towards the end of a generation, there may be no more orderings available
// for the generation, at the same time as a new generation cannot be created
// yet because other threads are still working on their orderings and have not
// submitted fitness scores for them. Since threads must obtain the lock both to
// reserve orderings and submit fitness scores, reserveOrdering() cannot simply
// hold the lock and wait until the remaining fitness scores have been received.
// In this situation, reserveOrdering() returns the invalid index of -1, at
// which point the caller must release the lock and wait a bit before trying
// again.

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
        self.population.organism_vec[ordering_idx].calc_connection_idx_vec()
    }

    pub fn release_ordering(
        &mut self,
        ordering_idx: OrderingIdx,
        n_completed_routes: usize,
        completed_route_cost: usize,
    ) {
        self.population.organism_vec[ordering_idx].n_completed_routes = n_completed_routes;
        self.population.organism_vec[ordering_idx].completed_route_cost = completed_route_cost;
        self.n_unprocessed_orderings -= 1;
    }
}
