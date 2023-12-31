use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use std::sync::{Arc, Mutex};

//
// Random
//

type GeneIdx = i32;

pub struct RandomIntGenerator {
    random_engine: ThreadRng,
    uniform_int_distribution: Uniform<i32>,
}

impl RandomIntGenerator {
    pub fn new() -> Self {
        Self {
            random_engine: thread_rng(),
            uniform_int_distribution: Uniform::new_inclusive(0, i32::MAX),
        }
    }

    pub fn with_range(min: i32, max: i32) -> Self {
        let mut rng = Self::new();
        rng.set_range(min, max);
        rng
    }

    pub fn set_range(&mut self, min: i32, max: i32) {
        self.uniform_int_distribution = Uniform::new_inclusive(min, max);
    }

    pub fn get_random_int(&mut self) -> i32 {
        self.uniform_int_distribution
            .sample(&mut self.random_engine)
    }
}

pub struct RandomFloatGenerator;

impl RandomFloatGenerator {
    pub fn get_normalized_random() -> f64 {
        let mut rng = thread_rng();
        let uniform = Uniform::new(0.0, 1.0);
        rng.sample(uniform)
    }
}

//
// Gene
//

type Gene = i32;
type GeneVec = Vec<Gene>;

pub struct GeneDependency {
    gene: Gene,
    gene_dependency: Gene,
}

impl GeneDependency {
    pub fn new(gene: Gene, gene_dependency: Gene) -> Self {
        Self {
            gene,
            gene_dependency,
        }
    }
}

//
// Organism
//

#[derive(Clone)]
pub struct Organism {
    n_genes: i32,
    random_gene_selector: Arc<Mutex<RandomIntGenerator>>,
    pub(crate) n_completed_routes: i32,
    pub(crate) completed_route_cost: i64,
    gene_vec: GeneVec,
}

impl Organism {
    pub fn new(n_genes: i32, random_gene_selector: Arc<Mutex<RandomIntGenerator>>) -> Self {
        Self {
            n_genes,
            random_gene_selector,
            n_completed_routes: 0,
            completed_route_cost: 0,
            gene_vec: Vec::new(),
        }
    }

    pub fn create_random(&mut self) {
        for _ in 0..self.n_genes {
            let mut random_gene_selector = self.random_gene_selector.lock().unwrap();
            self.gene_vec.push(random_gene_selector.get_random_int());
        }
    }

    pub fn get_random_crossover_point(&self) -> GeneIdx {
        let mut random_gene_selector = self.random_gene_selector.lock().unwrap();
        random_gene_selector.get_random_int()
    }

    pub fn mutate(&mut self) {
        let mut random_gene_selector = self.random_gene_selector.lock().unwrap();
        let dependent_idx = random_gene_selector.get_random_int() as usize;
        let dependency_idx = random_gene_selector.get_random_int();
        self.gene_vec[dependent_idx] = dependency_idx;
    }

    pub fn calc_connection_idx_vec(&self) -> GeneVec {
        let gene_vec = self.topo_sort();
        assert_eq!(gene_vec.len(), self.n_genes as usize);
        gene_vec
    }

    pub fn dump(&self) {
        print!(
            "nCompletedRoutes={} completedRouteCost={} nGenes={} genes=",
            self.n_completed_routes, self.completed_route_cost, self.n_genes
        );
        for v in &self.gene_vec {
            print!(" {}", v);
        }
        println!();
    }

    fn topo_sort(&self) -> GeneVec {
        let mut gene_list: Vec<GeneDependency> = self
            .gene_vec
            .iter()
            .enumerate()
            .map(|(i, &gene_idx)| GeneDependency::new(i as i32, gene_idx))
            .collect();

        gene_list.sort_by(|a, b| a.gene_dependency.cmp(&b.gene_dependency));

        let mut gene_vec = Vec::new();
        let mut dependency_set = std::collections::HashSet::new();

        while !gene_list.is_empty() {
            let mut found = false;
            let mut i = 0;
            while i != gene_list.len() {
                if dependency_set.contains(&gene_list[i].gene_dependency) {
                    gene_vec.push(gene_list[i].gene);
                    dependency_set.insert(gene_list[i].gene);
                    gene_list.remove(i);
                    found = true;
                } else {
                    i += 1;
                }
            }
            if !found {
                gene_vec.push(gene_list[0].gene);
                dependency_set.insert(gene_list[0].gene);
                gene_list.remove(0);
            }
        }
        gene_vec
    }
}

//
// Population
//

// #[derive(Clone)]
// pub struct OrganismPair<'a> {
//     a: &'a mut Organism,
//     b: &'a mut Organism,
// }
//
// impl<'a> OrganismPair<'a> {
//     pub fn new(a: &'a mut Organism, b: &'a mut Organism) -> Self {
//         Self { a, b }
//     }
// }

pub struct OrganismPair {
    a: Organism,
    b: Organism,
}

impl OrganismPair {
    pub fn new(a: Organism, b: Organism) -> Self {
        Self { a, b }
    }
}

type OrganismIdx = i32;
type OrganismVec = Vec<Organism>;

pub struct Population {
    pub(crate) n_organisms_in_population: i32,
    pub(crate) crossover_rate: f64,
    pub(crate) mutation_rate: f64,
    pub(crate) n_genes_per_organism: i32,
    pub(crate) random_gene_selector: Arc<Mutex<RandomIntGenerator>>,
    pub(crate) random_organism_selector: Arc<Mutex<RandomIntGenerator>>,
    pub(crate) organism_vec: OrganismVec,
}

impl Population {
    pub fn new(n_organisms_in_population: i32, crossover_rate: f64, mutation_rate: f64) -> Self {
        assert_eq!(n_organisms_in_population % 2, 0); // Must have even number of organisms
        Self {
            n_organisms_in_population,
            crossover_rate,
            mutation_rate,
            n_genes_per_organism: 0,
            random_gene_selector: Arc::new(Mutex::new(RandomIntGenerator::new())),
            random_organism_selector: Arc::new(Mutex::new(RandomIntGenerator::with_range(
                0,
                n_organisms_in_population - 1,
            ))),
            organism_vec: Vec::new(),
        }
    }

    pub fn reset(&mut self, n_genes_per_organism: i32) {
        self.n_genes_per_organism = n_genes_per_organism;
        self.random_gene_selector
            .lock()
            .unwrap()
            .set_range(0, n_genes_per_organism - 1);
        self.create_random_population();
    }

    pub fn next_generation(&mut self) {
        let mut new_generation_vec = Vec::new();
        let mut n_mutations = 0;
        for _ in 0..self.n_organisms_in_population / 2 {
            let crossover_rate = self.crossover_rate;
            let mutation_rate = self.mutation_rate;
            let mut pair = self.select_pair_tournament(2);
            if RandomFloatGenerator::get_normalized_random() < crossover_rate {
                // self.crossover(&mut pair);
                let cross_idx = pair.a.get_random_crossover_point() as usize;
                for i in cross_idx..pair.a.gene_vec.len() {
                    std::mem::swap(&mut pair.a.gene_vec[i], &mut pair.b.gene_vec[i]);
                }
            }
            if RandomFloatGenerator::get_normalized_random() < mutation_rate {
                pair.a.mutate();
                n_mutations += 1;
            }
            if RandomFloatGenerator::get_normalized_random() < mutation_rate {
                pair.b.mutate();
                n_mutations += 1;
            }
            new_generation_vec.push(pair.a);
            new_generation_vec.push(pair.b);
        }
        assert_eq!(
            new_generation_vec.len(),
            self.n_organisms_in_population as usize
        );
        self.organism_vec = new_generation_vec;
    }

    pub fn create_random_population(&mut self) {
        self.organism_vec.clear();
        for _ in 0..self.n_organisms_in_population {
            let mut organism = Organism::new(
                self.n_genes_per_organism,
                Arc::clone(&self.random_gene_selector),
            );
            organism.create_random();
            self.organism_vec.push(organism);
        }
    }

    // pub fn crossover(&mut self, pair: &mut OrganismPair) {
    //     let cross_idx = pair.a.get_random_crossover_point() as usize;
    //     for i in cross_idx..pair.a.gene_vec.len() {
    //         std::mem::swap(&mut pair.a.gene_vec[i], &mut pair.b.gene_vec[i]);
    //     }
    // }

    pub fn select_pair_tournament(&mut self, n_candidates: i32) -> OrganismPair {
        let organism_a_idx = self.tournament_select(n_candidates);
        loop {
            let organism_b_idx = self.tournament_select(n_candidates);
            if organism_a_idx != organism_b_idx {
                return OrganismPair::new(
                    self.organism_vec[organism_a_idx as usize].clone(),
                    self.organism_vec[organism_b_idx as usize].clone(),
                );
            }
        }
    }

    // A tournament in a genetic algorithm is a method of selection used to choose an individual
    // from a population. The idea is to select a few individuals at random from the population, and
    // then choose the best out of that small group.
    //
    // Here's how it works:
    //
    // 1. Decide on the tournament size. This is the number of individuals that will be randomly
    // selected from the population for the tournament.
    //
    // 2. Randomly select individuals from the population until you have enough to fill the
    // tournament.
    //
    // 3. Compare the fitness of all the individuals in the tournament, and select the one with the
    // best fitness.
    //
    // This process is repeated to select multiple individuals. The selected individuals are
    // typically used for crossover and mutation to form the next generation of the population.
    //
    // In the context of the provided code, the `tournament_select` function implements a tournament
    // selection. It takes `n_candidates` as a parameter, which is the tournament size. It then
    // selects `n_candidates` organisms from the population, compares their fitness (in this case,
    // the completed route cost), and returns the index of the organism with the best fitness.
    pub fn tournament_select(&self, n_candidates: i32) -> OrganismIdx {
        let mut best_organism_idx = -1;
        let mut lowest_completed_route_cost = i64::MAX;
        let mut n_highest_completed_routes = 0;

        for _ in 0..n_candidates {
            let organism_idx = self
                .random_organism_selector
                .lock()
                .unwrap()
                .get_random_int();

            let organism = &self.organism_vec[organism_idx as usize];

            let has_more_completed_routes =
                organism.n_completed_routes > n_highest_completed_routes;

            let has_equal_routes_and_lower_cost = organism.n_completed_routes
                == n_highest_completed_routes
                && organism.completed_route_cost < lowest_completed_route_cost;

            if has_more_completed_routes || has_equal_routes_and_lower_cost {
                best_organism_idx = organism_idx;
                lowest_completed_route_cost = organism.completed_route_cost;
                n_highest_completed_routes = organism.n_completed_routes;
            }
        }
        best_organism_idx
    }
}
