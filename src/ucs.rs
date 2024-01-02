use crate::layout::{Layout, RouteStepVec};
use crate::nets::Nets;
use crate::router::Router;
use crate::via::{CostVia, LayerCostVia, LayerVia, StartEndVia, ValidVia, Via, via_to_str};
use std::sync::Mutex;

use crate::board::Board;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use crate::settings::Settings;

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
        shortcut_end_via: Via,
    ) -> RouteStepVec {
        let end = start_end_via.end.clone_owned();
        let shortcut_end_via = Via::new(end.x, end.y);
        let found_route =
            self.find_costs(board, layout, nets, router, start_end_via, shortcut_end_via);
        self.dump_costs(board);
        return if found_route {
            self.backtrace_lowest_cost_route(
                board,
                layout,
                router,
                StartEndVia {
                    start: start_end_via.start,
                    end: shortcut_end_via,
                },
            )
        } else {
            RouteStepVec::new()
        };
    }

    fn find_costs(
        &mut self,
        board: Board,
        layout: &mut Layout,
        nets: &mut Nets,
        router: &mut Router,
        start_end_via: StartEndVia,
        shortcut_end_via: Via,
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
        while !self.frontier_pri.is_empty() {
            let node = self.frontier_pri.pop().unwrap();
            // println!("node: {:?}", node);
            let node_cost = node.0.cost;
            let layer_node = LayerVia::from_layer_cost_via(&node);
            // println!("layer_node: {:?}", layer_node);
            self.frontier_set.remove(&layer_node);
            let cur_cost = self.get_cost(board, layer_node);
            if layer_node.is_target(end.via) {
                return true;
            }
            self.explored_set.insert(layer_node);
            if layer_node.is_wire_layer {
                if layer_node.via.x > 0 {
                    self.explore_neighbour(board, layout, nets, router, layer_node, self.step_left(layer_node), start_end_via, shortcut_end_via, cur_cost, layout.settings.wire_cost);
                }
                if layer_node.via.x < board.w - 1 {
                    self.explore_neighbour(board, layout, nets, router, layer_node, self.step_right(layer_node), start_end_via, shortcut_end_via, cur_cost, layout.settings.wire_cost);
                }
                self.explore_neighbour(board, layout, nets, router, layer_node, self.step_to_strip(layer_node), start_end_via, shortcut_end_via, cur_cost, layout.settings.via_cost);
            } else {
                if layer_node.via.y > 0 {
                    self.explore_neighbour(board, layout, nets, router, layer_node, self.step_up(layer_node), start_end_via, shortcut_end_via, cur_cost, layout.settings.strip_cost);
                }
                if layer_node.via.y < board.h - 1 {
                    self.explore_neighbour(board, layout, nets, router, layer_node, self.step_down(layer_node), start_end_via, shortcut_end_via, cur_cost, layout.settings.strip_cost);
                }
                self.explore_neighbour(board, layout, nets, router, layer_node, self.step_to_wire(layer_node), start_end_via, shortcut_end_via, cur_cost, layout.settings.via_cost);
                let wire_to_via = router.wire_to_via_ref(board, layer_node.via);
                if wire_to_via.is_valid {
                    self.explore_frontier(board, layout, layer_node, LayerVia { via: wire_to_via.via, is_wire_layer: false }, cur_cost, layout.settings.wire_cost);
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
        node: LayerVia,
        n: LayerVia,
        start_end_via: StartEndVia,
        shortcut_end_via: Via,
        node_cost: usize,
        step_to_n_cost: usize,
    ) {
        if router.is_available(
            board,
            layout,
            nets,
            n,
            start_end_via.start,
            shortcut_end_via,
        ) {
            self.explore_frontier(board, layout, node, n, node_cost, step_to_n_cost);
        }
    }

    fn explore_frontier(
        &mut self,
        board: Board,
        layout: &mut Layout,
        node: LayerVia,
        n: LayerVia,
        node_cost: usize,
        step_to_n_cost: usize,
    ) {
        if self.explored_set.contains(&n) {
            return;
        }
        let cost = node_cost + step_to_n_cost;
        self.set_cost(board, n, cost);
        if !self.frontier_set.contains(&n) {
            self.frontier_pri.push(Reverse(LayerCostVia::from_layer_via(n, cost)));
            self.frontier_set.insert(n);
        } else {
            let frontier_cost = self.get_cost(board, n);
            if frontier_cost > cost {
                self.set_cost(board, node, cost);
            }
        }
    }

    fn backtrace_lowest_cost_route(
        &mut self,
        board: Board,
        layout: &mut Layout,
        router: &mut Router,
        start_end_via: StartEndVia,
    ) -> RouteStepVec {
        println!("start_end_via: {:?} {:?}", via_to_str(&start_end_via.start), via_to_str(&start_end_via.end));

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
                    "Error: backtraceLowestCostRoute() stuck at {}", c
                ));
                layout.diag_start_via = ValidVia::from_via(start.via);
                layout.diag_end_via = ValidVia::from_via(end.via);
                layout.diag_route_step_vec = route_step_vec.clone();
                layout.has_error = true;
                break;
            }

            let mut n = c.clone();

            println!("c: {:?} -> n: {:?}", c, n);;

            if c.is_wire_layer {
                if c.via.x > 0 {
                    let n_left = self.step_left(c);
                    if self.get_cost(board, n_left) < self.get_cost(board, n) {
                        n = n_left;
                    }
                }
                if c.via.x < board.w - 1 {
                    let n_right = self.step_right(c);
                    if self.get_cost(board, n_right) < self.get_cost(board, n) {
                        n = n_right;
                    }
                }
                let n_strip = self.step_to_strip(c);
                if self.get_cost(board, n_strip) < self.get_cost(board, n) {
                    n = n_strip;
                }
            } else {
                if c.via.y > 0 {
                    let n_up = self.step_up(c);
                    if self.get_cost(board, n_up) < self.get_cost(board, n) {
                        n = n_up;
                    }
                }
                if c.via.y < board.h - 1 {
                    let n_down = self.step_down(c);
                    if self.get_cost(board, n_down) < self.get_cost(board, n) {
                        n = n_down;
                    }
                }
                let n_wire = self.step_to_wire(c);
                if self.get_cost(board, n_wire) < self.get_cost(board, n) {
                    n = n_wire;
                }

                let wire_to_via = router.wire_to_via_ref(board, c.via);
                if wire_to_via.is_valid {
                    let mut n_wire_jump = LayerVia::from_via(wire_to_via.via, false);
                    if self.get_cost(board, n_wire_jump) < self.get_cost(board, n) {
                        route_step_vec.push(LayerVia::from_via(c.via, true));
                        let x1 = c.via.x;
                        let x2 = n_wire_jump.via.x;
                        if (x1 < x2) {
                            for x in (x1..x2) {
                                route_step_vec.push(LayerVia::from_via(Via::new(x, c.via.y), true));
                            }
                        } else {
                            for x in (x2..x1) {
                                route_step_vec.push(LayerVia::from_via(Via::new(x, c.via.y), true));
                            }
                        }
                        // The above is my replacement for this code that Copilot apparently screwed up on. It's not possible to step by -1.
                        // let step = if x1 > x2 { -1 } else { 1 };
                        // for x in (x1..x2).step_by(step) {
                        //     route_step_vec.push(LayerVia::from_via(Via::new(x, c.via.y), true));
                        // }
                        if x1 != x2 {
                            route_step_vec.push(LayerVia::from_via(Via::new(x2, c.via.y), true));
                        }
                        n = n_wire_jump;
                    }
                }
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
            println!("wire_cost: {:?} {:03x}", layer_via, self.via_cost_vec[i].wire_cost);
            self.via_cost_vec[i].wire_cost
        } else {
            println!("strip_cost: {:?} {:03x}", layer_via, self.via_cost_vec[i].strip_cost);
            self.via_cost_vec[i].strip_cost
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

    fn set_cost(&mut self, board: Board, layer_via: LayerVia, cost: usize) {
        let i = board.idx(layer_via.via);
        if layer_via.is_wire_layer {
            self.via_cost_vec[i].wire_cost = cost;
        } else {
            self.via_cost_vec[i].strip_cost = cost;
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
