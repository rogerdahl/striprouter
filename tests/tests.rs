            // if ui.button("Test GA").clicked() {
            //     // Test the GA by generating a sequence of numbers in random order
            //     // and telling the GA that costs are higher the more out of order
            //     // the ordering is. The GA should learn to generate orderings that
            //     // are in order.
            //     let mut g = GeneticAlgorithm::new(1000, 0.7, 0.01);
            //     g.reset(10);
            //     for _ in 0..100000 {
            //         let ordering_idx = g.reserve_ordering();
            //         let ordering = g.get_ordering(ordering_idx);
            //         println!("ordering={:?}", ordering);
            //         let mut cost = 0;
            //         for i in 0..ordering.len() {
            //             cost += ordering[i] * (i + 1);
            //         }
            //         g.release_ordering(ordering_idx, 10, cost);
            //     }
            // }
