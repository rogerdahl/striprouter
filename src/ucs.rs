use crate::layout::{Layout, RouteStepVec};
use crate::nets::Nets;
use crate::router::Router;
use crate::via::{CostVia, LayerCostVia, LayerVia, StartEndVia, ValidVia, Via};
use std::sync::Mutex;

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use crate::board::Board;

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
            via_cost_vec: vec![CostVia::new(); (board.size())],
            frontier_pri: BinaryHeap::new(),
            frontier_set: HashSet::new(),
            explored_set: HashSet::new(),
        }
    }

    pub fn find_lowest_cost_route(
        &mut self,
        board: Board,
        layout: &mut Layout,
        router: &mut Router,
        start_end_via: StartEndVia,
        shortcut_end_via: Via,
    ) -> RouteStepVec {
        let end = start_end_via.end.clone_owned();
        let shortcut_end_via = Via::new(end.x, end.y);
        let found_route = self.find_costs(board, router, start_end_via, shortcut_end_via);
        return if found_route {
            self.backtrace_lowest_cost_route(
                board,
                layout,
                StartEndVia {
                            start: start_end_via.start,
                            end: shortcut_end_via,
                        })
        } else {
            RouteStepVec::new()
        };
    }

    fn find_costs(
        &mut self,
        board: Board,
        // layout: &mut Layout,
        router: &mut Router,
        start_end_via: StartEndVia,
        shortcut_end_via: Via,
    ) -> bool {
        // let settings = &layout.settings;
        let start = LayerVia {
            via: start_end_via.start,
            is_wire_layer: false,
        };
        let end = LayerVia {
            via: start_end_via.end,
            is_wire_layer: false,
        };
        self.set_cost(board, LayerCostVia::from_layer_via(start, 0));
        self.frontier_pri.push(Reverse(LayerCostVia {
            layer_via: start,
            cost: 0,
        }));
        self.frontier_set.insert(LayerVia {
            via: start.via,
            is_wire_layer: false,
        });
        while !self.frontier_pri.is_empty() {
            let node = self.frontier_pri.pop().unwrap();
            self.frontier_set
                .remove(&LayerVia::from_layer_cost_via(&node));
            let node_cost = self.get_cost(board, node.0.layer_via);
            if node.0.layer_via.is_target(end.via) {
                return true;
            }
            self.explored_set.insert(node.0.layer_via);
            // if node.0.layer_via.is_wire_layer {

            // must add shortcut_end_via to all of these

            //     self.explore_neighbour(&mut node.0.clone(), self.step_left(node.0.layer_via));
            //     self.explore_neighbour(node, self.step_right(node.0.layer_via));
            //     self.explore_neighbour(node, self.step_to_strip(node.0.layer_via));
            // } else {
            //     self.explore_neighbour(node, self.step_up(node.0.layer_via));
            //     self.explore_neighbour(node, self.step_down(node.0.layer_via));
            //     self.explore_neighbour(node, self.step_to_wire(node.0.layer_via));
            //     let wire_to_via = self.router.wire_to_via_ref(node.0.layer_via);
            //     if wire_to_via.is_valid {
            //         self.explore_frontier(
            //             node,
            //             LayerVia {
            //                 via: wire_to_via.via,
            //                 is_wire_layer: false,
            //             },
            //         );
            //     }
            // }
        }
        false
    }

    fn explore_neighbour(
        &mut self,
        board: Board,
        layout: &mut Layout,
        router: &mut Router,
        node: LayerCostVia,
        n: LayerCostVia,
        start_end_via: StartEndVia,
        shortcut_end_via: Via,
    ) {
        if router.is_available(board, layout, n.layer_via, start_end_via.start, shortcut_end_via) {
            self.explore_frontier(board, layout, node, n);
        }
    }

    fn explore_frontier(&mut self, board: Board, layout: &mut Layout, node: LayerCostVia, mut n: LayerCostVia) {
        if self.explored_set.contains(&n.layer_via) {
            return;
        }
        n.cost += node.cost;
        self.set_cost(board, n);
        if !self.frontier_set.contains(&n.layer_via) {
            self.frontier_pri.push(Reverse(n.clone()));
            self.frontier_set.insert(n.layer_via.clone());
        } else {
            let frontier_cost = self.get_cost(board, n.layer_via);
            if frontier_cost > n.cost {
                // node.cost = n.cost;
                self.set_cost(board, node);
            }
        }
    }

    fn backtrace_lowest_cost_route(&mut self, board: Board, layout: &mut Layout, start_end_via: StartEndVia) -> RouteStepVec {
        let mut route_cost = 0;
        let start = LayerVia::from_via(start_end_via.start, false);
        let end = LayerVia::from_via(start_end_via.end, false);
        let mut route_step_vec = Vec::new();
        let mut c = end.clone();
        route_step_vec.push(c.clone());

        let mut check_stuck_cnt = 0;

        while c.via != start.via || c.is_wire_layer != start.is_wire_layer {
            if check_stuck_cnt > board.w * board.h {
                layout.error_string_vec.push(format!(
                    "Error: backtraceLowestCostRoute() stuck at {}",
                    c.str()
                ));
                layout.diag_start_via = ValidVia::from_via(start.via);
                layout.diag_end_via = ValidVia::from_via(end.via);
                layout.diag_route_step_vec = route_step_vec.clone();
                layout.has_error = true;
                break;
            }

            let mut n = c.clone();
            if c.is_wire_layer {
                let n_left = self.step_left(c);
                if c.via.x > 0 && self.get_cost(board, n_left) < self.get_cost(board, n) {
                    n = n_left;
                }
                let n_right = self.step_right(c);
                if c.via.x < board.w - 1 && self.get_cost(board, n_right) < self.get_cost(board, n) {
                    n = n_right;
                }
                let n_strip = self.step_to_strip(c);
                if self.get_cost(board, n_strip) < self.get_cost(board, n) {
                    n = n_strip;
                }
            } else {
                let n_up = self.step_up(c);
                if c.via.y > 0 && self.get_cost(board, n_up) < self.get_cost(board, n) {
                    n = n_up;
                }
                let n_down = self.step_down(c);
                if c.via.y < board.h - 1 && self.get_cost(board, n_down) < self.get_cost(board, n) {
                    n = n_down;
                }
                let n_wire = self.step_to_wire(c);
                if self.get_cost(board, n_wire) < self.get_cost(board,n) {
                    n = n_wire;
                }

                // let wire_to_via = self.router.wire_to_via_ref(c.via);
                // if wire_to_via.is_valid {
                //     let mut n_wire_jump = LayerVia::from_via(wire_to_via.via, false);
                //     if self.get_cost(n_wire_jump) < self.get_cost(n) {
                //         route_step_vec.push(LayerVia::from_via(c.via, true));
                //         let x1 = c.via.x;
                //         let x2 = n_wire_jump.via.x;
                //         if (x1 > x2) {
                //             for x in (x1..x2) {
                //                 route_step_vec.push(LayerVia::from_via(Via::new(x, c.via.y), true));
                //             }
                //         } else {
                //             for x in (x1..x2).rev() {
                //                 route_step_vec.push(LayerVia::from_via(Via::new(x, c.via.y), true));
                //             }
                //         }
                //         // The above is my replacement for this code that Copilot apparently screwed up on. It's not possible to step by -1.
                //         // let step = if x1 > x2 { -1 } else { 1 };
                //         // for x in (x1..x2).step_by(step) {
                //         //     route_step_vec.push(LayerVia::from_via(Via::new(x, c.via.y), true));
                //         // }
                //         if x1 != x2 {
                //             route_step_vec.push(LayerVia::from_via(Via::new(x2, c.via.y), true));
                //         }
                //         n = n_wire_jump;
                //     }
                // }
            }
            route_cost += self.get_cost(board, c) - self.get_cost(board, n);
            c = n;
            route_step_vec.push(c.clone());
        }
        layout.cost += route_cost;
        route_step_vec.reverse();

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
            self.via_cost_vec[i].wire_cost
        } else {
            self.via_cost_vec[i].strip_cost
        }
    }

    fn set_cost(&mut self, board: Board, layer_cost_via: LayerCostVia) {
        let i = board.idx(layer_cost_via.layer_via.via);
        if layer_cost_via.layer_via.is_wire_layer {
            self.via_cost_vec[i].wire_cost = layer_cost_via.cost;
        } else {
            self.via_cost_vec[i].strip_cost = layer_cost_via.cost;
        }
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
