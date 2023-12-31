use crate::layout::{Layout, RouteStepVec};
use crate::nets::Nets;
use crate::router::Router;
use crate::via::{CostVia, LayerCostVia, LayerVia, StartEndVia, ValidVia, Via};
use std::sync::Mutex;

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

type FrontierPri = BinaryHeap<Reverse<LayerCostVia>>;
type FrontierSet = HashSet<LayerVia>;
type ExploredSet = HashSet<LayerVia>;

pub struct UniformCostSearch<'a, 'b> {
    router: &'a Router<'b, 'b>,
    layout: &'a mut Layout,
    nets: &'a Nets<'a>,
    shortcut_end_via: &'a mut Via,
    via_start_end: StartEndVia,
    via_cost_vec: Vec<CostVia>,
    frontier_pri: FrontierPri,
    frontier_set: FrontierSet,
    explored_set: ExploredSet,
}

impl<'b, 'a> UniformCostSearch<'a, 'b> {
    pub fn new(
        router: &'a Router,
        layout: &'a mut Layout,
        nets: &'a Nets<'a>,
        shortcut_end_via: &'a mut Via,
        via_start_end: StartEndVia,
    ) -> Self {
        Self {
            router,
            layout,
            nets,
            shortcut_end_via,
            via_start_end,
            via_cost_vec: vec![CostVia::new(); (layout.grid_w * layout.grid_h) as usize],
            frontier_pri: BinaryHeap::new(),
            frontier_set: HashSet::new(),
            explored_set: HashSet::new(),
        }
    }

    pub fn find_lowest_cost_route(&mut self) -> RouteStepVec {
        self.shortcut_end_via = &mut self.via_start_end.end;
        let found_route = self.find_costs(self.shortcut_end_via);
        // #ifndef NDEBUG
        //   layout_.diagCostVec = viaCostVec_;
        // #endif
        return if found_route {
            self.backtrace_lowest_cost_route(StartEndVia {
                start: self.via_start_end.start,
                end: *self.shortcut_end_via,
            })
        } else {
            RouteStepVec::new()
        };
    }

    fn find_costs(&mut self, shortcut_end_via: &mut Via) -> bool {
        let settings = &self.layout.settings;
        let start = LayerVia {
            via: self.via_start_end.start,
            is_wire_layer: false,
        };
        let end = LayerVia {
            via: self.via_start_end.end,
            is_wire_layer: false,
        };
        self.set_cost(start, 0);
        self.frontier_pri.push(Reverse(LayerCostVia {
            layer_via: start,
            cost: 0,
        }));
        self.frontier_set.insert(LayerVia {
            via: start.via,
            is_wire_layer: false,
        });
        while !self.frontier_pri.is_empty() {
            //#ifndef NDEBUG
            //      layout_.errorStringVec.push_back(fmt::format("Debug:
            //      UniformCostSearch::findCosts() No route found"));
            //      layout_.diagStartVia = start.via;
            //      layout_.diagEndVia = end.via;
            //      layout_.hasError = true;
            //#endif
            let node = self.frontier_pri.pop().unwrap();
            self.frontier_set.remove(&node);
            let node_cost = self.get_cost(node.via);
            if self.router.is_target(node.via, end.via) {
                // #ifndef NDEBUG
                //     layout_.diagCostVec = viaCostVec_;
                // #endif
                return true;
            }
            self.explored_set.insert(node.via);
            if node.is_wire_layer {
                self.explore_neighbour(node, self.step_left(node.via));
                self.explore_neighbour(node, self.step_right(node.via));
                self.explore_neighbour(node, self.step_to_strip(node.via));
            } else {
                self.explore_neighbour(node, self.step_up(node.via));
                self.explore_neighbour(node, self.step_down(node.via));
                self.explore_neighbour(node, self.step_to_wire(node.via));
                let wire_to_via = self.router.wire_to_via_ref(node.via);
                if wire_to_via.is_valid {
                    self.explore_frontier(
                        node,
                        LayerVia {
                            via: wire_to_via.via,
                            is_wire_layer: false,
                        },
                    );
                }
            }
        }
        false
    }

    fn explore_neighbour(&mut self, node: &mut LayerCostVia, n: LayerCostVia) {
        if self.router.is_available(n.layer_via, self.via_start_end.start, *self.shortcut_end_via) {
            self.explore_frontier(node, n);
        }
    }

    fn explore_frontier(&mut self, node: &mut LayerCostVia, mut n: LayerCostVia) {
        if self.explored_set.contains(&n.layer_via) {
            return;
        }
        n.cost += node.cost;
        self.set_cost(n.layer_via, n.cost);
        if !self.frontier_set.contains(&n.layer_via) {
            self.frontier_pri.push(Reverse(n.clone()));
            self.frontier_set.insert(n.layer_via.clone());
        } else {
            let frontier_cost = self.get_cost(n.layer_via.clone());
            if frontier_cost > n.cost {
                node.cost = n.cost;
                self.set_cost(node.layer_via.clone(), node.cost);
            }
        }
    }

    fn backtrace_lowest_cost_route(&mut self, via_start_end: StartEndVia) -> RouteStepVec {
        let mut route_cost = 0;
        let start = LayerVia::from_via(via_start_end.start, false);
        let end = LayerVia::from_via(via_start_end.end, false);
        let mut route_step_vec = Vec::new();
        let mut c = end.clone();
        route_step_vec.push(c.clone());

        let mut check_stuck_cnt = 0;

        while c.via != start.via || c.is_wire_layer != start.is_wire_layer {
            if check_stuck_cnt > self.layout.grid_w * self.layout.grid_h {
                self.layout.error_string_vec.push(format!("Error: backtraceLowestCostRoute() stuck at {}", c.str()));
                self.layout.diag_start_via = ValidVia::from_via(start.via, true);
                self.layout.diag_end_via = ValidVia::from_via(end.via, true);
                self.layout.diag_route_step_vec = route_step_vec.clone();
                self.layout.has_error = true;
                break;
            }

            let mut n = c.clone();
            if c.is_wire_layer {
                let n_left = self.step_left(c.via);
                if c.via.x > 0 && self.get_cost(n_left) < self.get_cost(n) {
                    n = n_left;
                }
                let n_right = self.step_right(c.via);
                if c.via.x < self.layout.grid_w - 1 && self.get_cost(n_right) < self.get_cost(n) {
                    n = n_right;
                }
                let n_strip = self.step_to_strip(c.via);
                if self.get_cost(n_strip) < self.get_cost(n) {
                    n = n_strip;
                }
            } else {
                let n_up = self.step_up(c.via);
                if c.via.y > 0 && self.get_cost(n_up) < self.get_cost(n) {
                    n = n_up;
                }
                let n_down = self.step_down(c.via);
                if c.via.y < self.layout.grid_h - 1 && self.get_cost(n_down) < self.get_cost(n) {
                    n = n_down;
                }
                let n_wire = self.step_to_wire(c.via);
                if self.get_cost(n_wire) < self.get_cost(n) {
                    n = n_wire;
                }

                let wire_to_via = self.router.wire_to_via_ref(c.via);
                if wire_to_via.is_valid {
                    let mut n_wire_jump = LayerVia::from_via(wire_to_via.via, false);
                    if self.get_cost(n_wire_jump) < self.get_cost(n) {
                        route_step_vec.push(LayerVia::from_via(c.via, true));
                        let x1 = c.via.x;
                        let x2 = n_wire_jump.via.x;
                        let step = if x1 > x2 { -1 } else { 1 };
                        for x in (x1..x2).step_by(step) {
                            route_step_vec.push(LayerVia::from_via(Via::new(x, c.via.y), true));
                        }
                        if x1 != x2 {
                            route_step_vec.push(LayerVia::from_via(Via::new(x2, c.via.y), true));
                        }
                        n = n_wire_jump;
                    }
                }
            }
            route_cost += self.get_cost(c) - self.get_cost(n);
            c = n;
            route_step_vec.push(c.clone());
        }
        self.layout.cost += route_cost;
        route_step_vec.reverse();

        #[cfg(debug_assertions)]
        {
            self.layout.diag_route_step_vec = route_step_vec.clone();
            self.layout.diag_start_via = ValidVia::from_via(start.via, true);
            self.layout.diag_end_via = ValidVia::from_via(end.via, true);
        }

        route_step_vec
    }

    fn get_cost(&self, via_layer: LayerVia) -> i32 {
        let i = self.layout.idx(&via_layer.via) as usize;
        if via_layer.is_wire_layer {
            self.via_cost_vec[i].wire_cost
        } else {
            self.via_cost_vec[i].strip_cost
        }
    }

    fn set_cost(&mut self, via_layer: LayerVia, cost: i32) {
        let i = self.layout.idx(&via_layer.via) as usize;
        if via_layer.is_wire_layer {
            self.via_cost_vec[i].wire_cost = cost;
        } else {
            self.via_cost_vec[i].strip_cost = cost;
        }
    }

    // fn set_cost(&mut self, via_layer_cost: LayerCostVia) {
    //     self.set_cost(via_layer_cost.layer_via, via_layer_cost.cost);
    // }

    fn step_left(&self, v: LayerVia) -> LayerVia {
        LayerVia::from_via(v.via + Via::new(-1, 0), v.is_wire_layer)
    }

    fn step_right(&self, v: LayerVia) -> LayerVia {
        LayerVia::from_via(v.via + Via::new(1, 0), v.is_wire_layer)
    }

    fn step_up(&self, v: LayerVia) -> LayerVia {
        LayerVia::from_via(v.via + Via::new(0, -1), v.is_wire_layer)
    }

    fn step_down(&self, v: LayerVia) -> LayerVia {
        LayerVia::from_via(v.via + Via::new(0, 1), v.is_wire_layer)
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
