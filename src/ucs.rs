use crate::layout::{Layout, RouteStepVec};
use crate::nets::Nets;
use crate::router::Router;
use crate::via::{via_to_str, CostVia, LayerCostVia, LayerVia, StartEndVia, ValidVia, Via};
use std::sync::Mutex;

use crate::board::Board;
use crate::settings::Settings;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

type FrontierPri = BinaryHeap<Reverse<LayerCostVia>>;
type FrontierSet = HashSet<LayerVia>;
type ExploredSet = HashSet<LayerVia>;

pub struct UniformCostSearch {
    via_cost_vec: Vec<CostVia>,
    frontier_pri: FrontierPri,
    frontier_set: FrontierSet,
    explored_set: ExploredSet,
}

impl UniformCostSearch {
    pub fn new(board: Board) -> Self {
        Self {
            via_cost_vec: vec![CostVia::new(); board.size()],
            frontier_pri: BinaryHeap::new(),
            frontier_set: HashSet::new(),
            explored_set: HashSet::new(),
        }
    }

    pub fn find_lowest_cost_route(
        &mut self,
        board: Board,
        layout: &mut Layout,
        nets: &mut Nets,
        router: &mut Router,
        // settings: &mut Settings,
        start_end_via: StartEndVia,
    ) -> RouteStepVec {
        let end = start_end_via.end.clone_owned();
        let found_route = self.find_costs(board, layout, nets, router, start_end_via);

        // self.dump_costs(board);

        return if found_route {
            self.backtrace_lowest_cost_route(
                board,
                layout,
                router,
                start_end_via
            )
        } else {
            RouteStepVec::new()
        };
    }

    // 'procedure' 'UniformCostSearch'(Graph, start, goal)
    //   node ← start
    //   cost ← 0
    //   frontier ← priority queue containing node only
    //   explored ← empty set
    //   'do'
    //     'if' frontier is empty
    //       'return' failure
    //     node ← frontier.pop()
    //     'if' node is goal
    //       'return' layout
    //     explored.add(node)
    //     'for each' of node's neighbors n
    //       'if' n is not in explored
    //         'if' n is not in frontier
    //           frontier.add(n)
    //         'else if' n is in frontier with higher cost
    //           replace existing node with n

    fn find_costs(
        &mut self,
        board: Board,
        layout: &mut Layout,
        nets: &mut Nets,
        router: &mut Router,
        start_end_via: StartEndVia,
    ) -> bool {
        let start = LayerVia {
            via: start_end_via.start,
            is_wire_layer: false,
        };
        let end = LayerVia {
            via: start_end_via.end,
            is_wire_layer: false,
        };
        self.set_cost(board, start, 0);
        self.frontier_pri.push(Reverse(LayerCostVia {
            layer_via: start,
            cost: 0,
        }));
        self.frontier_set.insert(LayerVia {
            via: start.via,
            is_wire_layer: false,
        });
        // #[rustfmt::skip]
        while !self.frontier_pri.is_empty() {
            let node = self.frontier_pri.pop().unwrap();

            // LOTS OF COST TWEAKS HERE

            let layer_node = LayerVia::from_layer_cost_via(&node);
            // println!("layer_node: {:?}", layer_node);
            assert!(self.frontier_set.remove(&layer_node));

            if layer_node.is_target(end.via) {
                return true;
            }

            self.explored_set.insert(layer_node);

            if layer_node.is_wire_layer {
                if layer_node.via.x > 0 {
                    self.explore_neighbour(board, layout, nets, router, layer_node, self.step_left(layer_node), start_end_via, layout.settings.wire_cost);
                }
                if layer_node.via.x < board.w - 1 {
                    self.explore_neighbour(board, layout, nets, router, layer_node, self.step_right(layer_node), start_end_via, layout.settings.wire_cost);
                }
                self.explore_neighbour(board, layout, nets, router, layer_node, self.step_to_strip(layer_node), start_end_via, layout.settings.via_cost);
            } else {
                if layer_node.via.y > 0 {
                    self.explore_neighbour(board, layout, nets, router, layer_node, self.step_up(layer_node), start_end_via, layout.settings.strip_cost);
                }
                if layer_node.via.y < board.h - 1 {
                    self.explore_neighbour(board, layout, nets, router, layer_node, self.step_down(layer_node), start_end_via, layout.settings.strip_cost);
                }
                self.explore_neighbour(board, layout, nets, router, layer_node, self.step_to_wire(layer_node), start_end_via, layout.settings.via_cost);

                // Wire jumps
                let wire_to_via = router.wire_to_via_ref(board, layer_node.via);
                if wire_to_via.is_valid {
                    self.explore_frontier(board, layout, layer_node, LayerVia { via: wire_to_via.via, is_wire_layer: false }, layout.settings.wire_cost);
                }
            }
        }
        false
    }

    fn explore_neighbour(
        &mut self,
        board: Board,
        layout: &mut Layout,
        nets: &mut Nets,
        router: &mut Router,
        cur_node: LayerVia,
        next_node: LayerVia,
        start_end_via: StartEndVia,
        step_cost: usize,
    ) {
        if router.is_available(board, layout, nets, next_node, start_end_via.start) {
            self.explore_frontier(board, layout, cur_node, next_node, step_cost);
        }
    }

    fn explore_frontier(
        &mut self,
        board: Board,
        layout: &mut Layout,
        cur_node: LayerVia,
        next_node: LayerVia,
        step_cost: usize,
    ) {
        // println!("--");
        // println!("cur_node={:?}", cur_node);
        // println!("next_node={:?}", next_node);

        if self.explored_set.contains(&next_node) {
            return;
        }

        let next_node_cost = self.get_cost(board, cur_node) + step_cost;

        if !self.frontier_set.contains(&next_node) {
            self.frontier_set.insert(next_node);
            // assert_eq!(self.get_cost(board, next_node), 0);
            // println!("next_node={:?} get_cost()={}", next_node, self.get_cost(board, next_node));
            self.set_cost(board, next_node, next_node_cost);
            // println!("SET next_node={} next_node_cost={}", next_node, next_node_cost);
            self.frontier_pri.push(Reverse(LayerCostVia::from_layer_via(next_node, next_node_cost)));
        } else {
            // If frontier_set contains next_node, we may already have visited
            // next_node, via another route. If so, the route we're reaching it by now
            // may have a lower cost, in which case we update it with the lower cost. If
            // we haven't visited next_node before, it will be set to usize::MAX, so it
            // will always be updated with the lower cost.
            let frontier_cost = self.get_cost(board, next_node);
            // println!("GET next_node={} frontier_cost={}", next_node, frontier_cost);
            if frontier_cost > next_node_cost {
                self.set_cost(board, next_node, next_node_cost);
            }
        }
    }

    // fn backtrace_lowest_cost_route(
    //     &mut self,
    //     board: Board,
    //     layout: &mut Layout,
    //     router: &mut Router,
    //     start_end_via: StartEndVia,
    // ) -> RouteStepVec {
    //     // println!("start_end_via: {:?} {:?}", via_to_str(&start_end_via.start), via_to_str(&start_end_via.end));
    //
    //     let mut route_cost = 0;
    //     let start = LayerVia::from_via(start_end_via.start, false);
    //     let end = LayerVia::from_via(start_end_via.end, false);
    //     let mut route_step_vec = Vec::new();
    //
    //     // We start at the end and work backwards to the start.
    //     let mut cur_node = end.clone();
    //
    //     route_step_vec.push(cur_node.clone());
    //
    //     let mut check_stuck_cnt = 0;
    //
    //     while cur_node.via != start.via || cur_node.is_wire_layer != start.is_wire_layer {
    //         check_stuck_cnt += 1;
    //         // If this assert fails, the backtrace has become stuck in an infinite loop.
    //         // This indicates a bug in find_cost(). The dump_costs() method will
    //         // probably show that the current location is one which does not have any
    //         // lower cost neighbours.
    //         assert!(check_stuck_cnt < board.w * board.h);
    //
    //         let mut next_node = cur_node.clone();
    //         // println!("c: {:?}", c);;
    //
    //         if cur_node.is_wire_layer {
    //             if cur_node.via.x > 0 {
    //                 let n_left = self.step_left(cur_node);
    //                 if self.get_cost(board, n_left) < self.get_cost(board, next_node) {
    //                     next_node = n_left;
    //                 }
    //             }
    //             if cur_node.via.x < board.w - 1 {
    //                 let n_right = self.step_right(cur_node);
    //                 if self.get_cost(board, n_right) < self.get_cost(board, next_node) {
    //                     next_node = n_right;
    //                 }
    //             }
    //             let n_strip = self.step_to_strip(cur_node);
    //             if self.get_cost(board, n_strip) < self.get_cost(board, next_node) {
    //                 next_node = n_strip;
    //             }
    //         } else {
    //             if cur_node.via.y > 0 {
    //                 let n_up = self.step_up(cur_node);
    //                 if self.get_cost(board, n_up) < self.get_cost(board, next_node) {
    //                     next_node = n_up;
    //                 }
    //             }
    //             if cur_node.via.y < board.h - 1 {
    //                 let n_down = self.step_down(cur_node);
    //                 if self.get_cost(board, n_down) < self.get_cost(board, next_node) {
    //                     next_node = n_down;
    //                 }
    //             }
    //             let n_wire = self.step_to_wire(cur_node);
    //             if self.get_cost(board, n_wire) < self.get_cost(board, next_node) {
    //                 next_node = n_wire;
    //             }
    //
    //             let wire_to_via = router.wire_to_via_ref(board, cur_node.via);
    //             if wire_to_via.is_valid {
    //                 let mut n_wire_jump = LayerVia::from_via(wire_to_via.via, false);
    //                 if self.get_cost(board, n_wire_jump) < self.get_cost(board, next_node) {
    //                     route_step_vec.push(LayerVia::from_via(cur_node.via, true));
    //                     let x1 = cur_node.via.x;
    //                     let x2 = n_wire_jump.via.x;
    //                     if (x1 < x2) {
    //                         for x in (x1..x2) {
    //                             route_step_vec.push(LayerVia::from_via(Via::new(x, cur_node.via.y), true));
    //                         }
    //                     } else {
    //                         for x in (x2..x1) {
    //                             route_step_vec.push(LayerVia::from_via(Via::new(x, cur_node.via.y), true));
    //                         }
    //                     }
    //                     // The above is my replacement for this code that Copilot apparently screwed up on. It's not possible to step by -1.
    //                     // let step = if x1 > x2 { -1 } else { 1 };
    //                     // for x in (x1..x2).step_by(step) {
    //                     //     route_step_vec.push(LayerVia::from_via(Via::new(x, c.via.y), true));
    //                     // }
    //                     if x1 != x2 {
    //                         route_step_vec.push(LayerVia::from_via(Via::new(x2, cur_node.via.y), true));
    //                     }
    //                     next_node = n_wire_jump;
    //                 }
    //             }
    //         }
    //         route_cost += self.get_cost(board, cur_node) - self.get_cost(board, next_node);
    //         cur_node = next_node;
    //         route_step_vec.push(cur_node.clone());
    //     }
    //     layout.cost += route_cost;
    //     route_step_vec.reverse();
    //
    //     #[cfg(debug_assertions)]
    //     {
    //         layout.diag_route_step_vec = route_step_vec.clone();
    //         layout.diag_start_via = ValidVia::from_via(start.via);
    //         layout.diag_end_via = ValidVia::from_via(end.via);
    //     }
    //
    //     route_step_vec
    // }

    fn backtrace_lowest_cost_route(
        &mut self,
        board: Board,
        layout: &mut Layout,
        router: &mut Router,
        start_end_via: StartEndVia,
    ) -> RouteStepVec {
        let mut route_cost = 0;
        let start = LayerVia::from_via(start_end_via.start, false);
        let end = LayerVia::from_via(start_end_via.end, false);
        let mut route_step_vec = Vec::new();
        let mut cur_node = end.clone();

        route_step_vec.push(cur_node.clone());

        let mut check_stuck_cnt = 0;

        while cur_node.via != start.via || cur_node.is_wire_layer != start.is_wire_layer {
            check_stuck_cnt += 1;
            assert!(check_stuck_cnt < board.w * board.h);

            let mut next_node = cur_node.clone();

            // TODO: Surely, all these should be if-else? And don't call me Shirley.
            if cur_node.is_wire_layer {
                if cur_node.via.x > 0 {
                    let n_left = self.step_left(cur_node);
                    if self.get_cost(board, n_left) < self.get_cost(board, next_node) {
                        next_node = n_left;
                    }
                }
                if cur_node.via.x < board.w - 1 {
                    let n_right = self.step_right(cur_node);
                    if self.get_cost(board, n_right) < self.get_cost(board, next_node) {
                        next_node = n_right;
                    }
                }
                let n_strip = self.step_to_strip(cur_node);
                if self.get_cost(board, n_strip) < self.get_cost(board, next_node) {
                    next_node = n_strip;
                }
            } else {
                if cur_node.via.y > 0 {
                    let n_up = self.step_up(cur_node);
                    if self.get_cost(board, n_up) < self.get_cost(board, next_node) {
                        next_node = n_up;
                    }
                }
                if cur_node.via.y < board.h - 1 {
                    let n_down = self.step_down(cur_node);
                    if self.get_cost(board, n_down) < self.get_cost(board, next_node) {
                        next_node = n_down;
                    }
                }
                let n_wire = self.step_to_wire(cur_node);
                if self.get_cost(board, n_wire) < self.get_cost(board, next_node) {
                    next_node = n_wire;
                }

                let wire_to_via = router.wire_to_via_ref(board, cur_node.via);
                if wire_to_via.is_valid {
                    let mut n_wire_jump = LayerVia::from_via(wire_to_via.via, false);
                    if self.get_cost(board, n_wire_jump) < self.get_cost(board, next_node) {
                        // When we jump, we have to record the steps.
                        // Through to wire layer.
                        route_step_vec.push(LayerVia::from_via(cur_node.via, true));
                        let x1 = cur_node.via.x;
                        let x2 = n_wire_jump.via.x;
                        if x1 > x2 {
                            for x in (x2..x1).rev() {
                                route_step_vec.push(LayerVia::from_via(Via::new(x, cur_node.via.y), true));
                            }
                        } else {
                            for x in (x1..x2) {
                                route_step_vec.push(LayerVia::from_via(Via::new(x, cur_node.via.y), true));
                            }
                        }
                        // let step = if x1 > x2 { -1 } else { 1 };
                        // for x in (x1..x2).step_by(step) {
                        //     route_step_vec.push(LayerVia::from_via(Via::new(x, cur_node.via.y), true));
                        // }
                        if x1 != x2 {
                            route_step_vec.push(LayerVia::from_via(Via::new(x2, cur_node.via.y), true));
                        }
                        // Final step through to strip layer is stored outside the
                        // conditional.
                        next_node = n_wire_jump;
                    }
                }
            }
            route_cost += self.get_cost(board, cur_node) - self.get_cost(board, next_node);
            // println!("route_cost={}", route_cost);
            cur_node = next_node;
            route_step_vec.push(cur_node.clone());
        }

        layout.cost += route_cost;
        // route_step_vec.reverse();

        #[cfg(debug_assertions)]
        {
            layout.diag_route_step_vec = route_step_vec.clone();
            layout.diag_start_via = ValidVia::from_via(start.via);
            layout.diag_end_via = ValidVia::from_via(end.via);
        }

        route_step_vec
    }

    fn get_cost(&self, board: Board, layer_via: LayerVia) -> usize {
        let i = board.idx(layer_via.via);
        if layer_via.is_wire_layer {
            // println!("wire_cost: {:?} {:03x}", layer_via, self.via_cost_vec[i].wire_cost);
            self.via_cost_vec[i].wire_cost
        } else {
            // println!("strip_cost: {:?} {:03x}", layer_via, self.via_cost_vec[i].strip_cost);
            self.via_cost_vec[i].strip_cost
        }
    }

    fn set_cost(&mut self, board: Board, layer_via: LayerVia, cost: usize) {
        let i = board.idx(layer_via.via);
        if layer_via.is_wire_layer {
            self.via_cost_vec[i].wire_cost = cost;
        } else {
            self.via_cost_vec[i].strip_cost = cost;
        }
    }

    fn dump_n(&self, v: usize) {
        if v == usize::MAX {
            print!("--- ");
        } else if v == 0 {
            print!("/// ");
        } else {
            print!("{:03x} ", if v > 0xfff { 0xfff } else { v });
        }
    }

    fn dump_costs(&self, board: Board) {
        println!("###### Costs:");
        println!("## Wire");
        print!("    ");
        for x in 0..board.w {
            print!("{:03x} ", x);
        }
        println!();
        for y in 0..board.h {
            print!("{:03x} ", y);
            for x in 0..board.w {
                let i = board.idx(Via::new(x, y));
                let v = self.via_cost_vec[i].wire_cost;
                // assert_eq!(v, self.get_cost(board, LayerVia::from_via(Via::new(x, y), true)));
                self.dump_n(v);
            }
            println!();
        }
        println!();
        println!("## Strip");
        print!("    ");
        for x in 0..board.w {
            print!("{:03x} ", x);
        }
        println!();
        for y in 0..board.h {
            print!("{:03x} ", y);
            for x in 0..board.w {
                let i = board.idx(Via::new(x, y));
                let v = self.via_cost_vec[i].strip_cost;
                // assert_eq!(v, self.get_cost(board, LayerVia::from_via(Via::new(x, y), false)));
                self.dump_n(v);
            }
            println!();
        }
        println!();
    }

    fn step_left(&self, v: LayerVia) -> LayerVia {
        LayerVia::from_via(Via::new(v.via.x - 1, v.via.y), v.is_wire_layer)
    }

    fn step_right(&self, v: LayerVia) -> LayerVia {
        LayerVia::from_via(Via::new(v.via.x + 1, v.via.y), v.is_wire_layer)
    }

    fn step_up(&self, v: LayerVia) -> LayerVia {
        LayerVia::from_via(Via::new(v.via.x, v.via.y - 1), v.is_wire_layer)
    }

    fn step_down(&self, v: LayerVia) -> LayerVia {
        LayerVia::from_via(Via::new(v.via.x, v.via.y + 1), v.is_wire_layer)
    }

    fn step_to_wire(&self, v: LayerVia) -> LayerVia {
        assert!(!v.is_wire_layer);
        LayerVia::from_via(v.via, true)
    }

    fn step_to_strip(&self, v: LayerVia) -> LayerVia {
        assert!(v.is_wire_layer);
        LayerVia::from_via(v.via, false)
    }
}
