use crate::board::Board;
use crate::layout::Layout;
use crate::nets::Nets;
// use crate::thread_stop::ThreadStop;
use crate::ucs::UniformCostSearch;
use crate::via::{LayerStartEndVia, LayerVia, StartEndVia, ValidVia, Via, WireLayerVia};

use std::collections::HashSet;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

pub struct Router {
    board: Board,
    // connection_idx_vec: Vec<usize>,
    all_pin_set: HashSet<Via>,
    via_trace_vec: Vec<WireLayerVia>,
    // input_layout: & Layout,
    // current_layout: & Layout,
    // thread_stop: & ThreadStop,
    // max_render_delay: Duration,
}

impl Router {
    pub fn new(board: Board) -> Self {
        Self {
            board,
            // connection_idx_vec: Vec::new(),
            all_pin_set: HashSet::new(),
            // THIS FILLS WITH 0,0 VIAS WHILE THE C++ VERSION FILLS WITH -1,-1.
            via_trace_vec: vec![WireLayerVia::new(); board.size()],
        }
    }

    pub fn route(
        &mut self,
        board: Board,
        layout: &mut Layout,
        nets: &mut Nets,
        connection_idx_vec: Vec<usize>,
        limit_routes: &mut Arc<AtomicUsize>,
    ) -> bool {
        self.block_component_footprints(board, layout);
        self.join_all_connections(board, layout, nets);
        self.register_active_component_pins(layout);
        let is_aborted = self.route_all(board, layout, nets, connection_idx_vec, limit_routes);
        let strip_cut_vec = self.find_strip_cuts(board, layout, nets);

        // TODO: Renable!
        // layout.cost += (layout.settings.cut_cost * strip_cut_vec.len());

        // if layout.has_error {
        //     layout.diag_trace_vec = self.via_trace_vec.clone();
        // }

        is_aborted
    }

    fn route_all(
        &mut self,
        board: Board,
        layout: &mut Layout,
        nets: &mut Nets,
        connection_idx_vec: Vec<usize>,
        limit_routes: &mut Arc<AtomicUsize>,
    ) -> bool {
        // TODO: remove, was testing
        // let mut nets = Nets::new(board);
        // nets.clear(board);
        // self.join_all_connections(board, layout, nets);

        let mut is_aborted = false;
        let start_time = Instant::now();
        let connection_via_vec = layout.circuit.gen_connection_via_vec();
        layout.route_status_vec.resize(connection_via_vec.len(), false);

        let mut limit = limit_routes.load(std::sync::atomic::Ordering::Relaxed);
        // println!("inner router: limit_routes = {}", limit);

        for connection_idx in connection_idx_vec.clone() {
            if limit == 0 {
                break;
            }

            limit -= 1;

            let start_end_via = connection_via_vec[connection_idx];
            let route_was_found = self.find_complete_route(board, layout, nets, start_end_via);
            layout.route_status_vec[connection_idx] = route_was_found;
            // if self.thread_stop.is_stopped() {
            //     is_aborted = true;
            //     break;
            // }
            // {
            //     let _lock: MutexGuard<_> = self.input_layout.scope_lock().unwrap();
            //     if !layout.is_based_on(&self.input_layout) {
            //         is_aborted = true;
            //         break;
            //     }
            // }
            // if layout.has_error {
            //     break;
            // }

            // if start_time.elapsed() > self.max_render_delay {
            //     let _lock: MutexGuard<_> = self.current_layout.scope_lock().unwrap();
            //     self.current_layout = layout;
            //     // start_time = Instant::now();
            // }
        }
        is_aborted
    }

    // There are two main approaches possible when routing with potential shortcut.
    //
    // (1) If, when routing from A to B, the router starts at A, finds a shortcut to
    // B, and routes only to the shortcut, B remains unconnected. It then becomes
    // necessary to do a second route, starting at B, and routing to A or a shortcut
    // to A.
    //
    // (2) However, if Uniform Cost Search is instead allowed to flow through
    // shortcuts but does not stop there, one gets a route that always connects A
    // and B, but will follow low cost routes along shortcuts when possible.
    //
    // I've currently implemented (2). I'm not sure if (1) would create any
    // different routes.

    fn find_complete_route(
        &mut self,
        board: Board,
        layout: &mut Layout,
        nets: &mut Nets,
        start_end_via: StartEndVia,
    ) -> bool {
        let route_was_found = self.find_route(board, layout, nets, start_end_via);
        if route_was_found {
            layout.n_completed_routes += 1;
        } else {
            layout.n_failed_routes += 1;
        }
        route_was_found
    }

    fn find_route(&mut self, board: Board, layout: &mut Layout, nets: &mut Nets, start_end_via: StartEndVia) -> bool {
        let mut ucs = UniformCostSearch::new(board);
        let route_step_vec = ucs.find_lowest_cost_route(board, layout, nets, self, start_end_via);
        if route_step_vec.is_empty() {
            return false;
        }
        // if layout.circuit.has_parser_error() || route_step_vec.is_empty() {
        //     return false;
        // }
        self.block_route(board, route_step_vec.clone());
        nets.connect_route(board, layout, &route_step_vec);
        let route_section_vec = self.condense_route(route_step_vec);
        self.add_wire_jumps(board, route_section_vec.clone());
        layout.route_vec.push(route_section_vec);
        true
    }

    // - Route always starts and ends on wire layer.
    // - Through to wire always starts a wire section.
    // - Through to strip always ends a wire section.
    // - Everything else is a strip section.
    fn condense_route(&self, route_step_vec: Vec<LayerVia>) -> Vec<LayerStartEndVia> {
        let mut route_section_vec = Vec::new();
        assert!(!route_step_vec[0].is_wire_layer);
        assert!(!route_step_vec.last().unwrap().is_wire_layer);
        let mut start_section = route_step_vec[0].clone();
        for i in 1..route_step_vec.len() {
            if route_step_vec[i].is_wire_layer != route_step_vec[i - 1].is_wire_layer {
                if i - 1 != 0 {
                    route_section_vec.push(LayerStartEndVia {
                        start: start_section,
                        end: route_step_vec[i - 1].clone(),
                    });
                    start_section = route_step_vec[i].clone();
                }
            }
        }
        if start_section != *route_step_vec.last().unwrap() {
            route_section_vec.push(LayerStartEndVia {
                start: start_section,
                end: route_step_vec.last().unwrap().clone(),
            });
        }
        route_section_vec
    }

    // Transitions
    // Cuts at:
    // - used <-> other used
    // - used <-> other pin
    // Cuts NOT at:
    // - unused <-> used
    // - unused <-> pin
    // - used <-> same pin
    fn find_strip_cuts(&self, board: Board, layout: &mut Layout, nets: &mut Nets) -> Vec<Via> {
        let mut v = Vec::new();
        for x in 0..self.board.w {
            let mut is_used = false;
            for y in 1..board.h {
                let prev_via = Via::new(x, y - 1);
                let cur_via = Via::new(x, y);
                let is_connected = nets.is_connected(board, layout, cur_via, prev_via);
                let is_in_other_net = nets.has_connection(board, layout, cur_via) && !is_connected;
                let is_other_pin = self.is_any_pin(cur_via) && !is_connected;
                if is_in_other_net || is_other_pin {
                    if is_used {
                        v.push(cur_via);
                    }
                    is_used = true;
                }
            }
        }
        v
    }

    //
    // Interface for Uniform Cost Search
    //

    pub fn is_available(
        &self,
        board: Board,
        layout: &mut Layout,
        nets: &mut Nets,
        cur_node: LayerVia,
        start_node: Via,
    ) -> bool {
        if cur_node.via.x >= board.w || cur_node.via.y >= board.h {
            return false;
        }
        if cur_node.is_wire_layer {
            if self.is_blocked(board, cur_node.via) {
                return false;
            }
        } else {
            // If it has an equivalent, it must be our equivalent
            if nets.has_connection(board, layout, cur_node.via)
                && !nets.is_connected(board, layout, cur_node.via, start_node)
            {
                return false;
            }
            // Can go to component pin only if it's our equivalent.
            if self.is_any_pin(cur_node.via) {
                if !nets.is_connected(board, layout, cur_node.via, start_node) {
                    return false;
                }
            }
        }
        // Can go there!
        true
    }

    fn is_any_pin(&self, via: Via) -> bool {
        self.all_pin_set.contains(&via)
    }

    pub fn wire_to_via_ref(&mut self, board: Board, via: Via) -> &mut ValidVia {
        let i = board.idx(via);
        &mut self.via_trace_vec[i].wire_to_via
    }

    //
    // Wire layer blocking
    //

    // Block the entire component footprint on the wire layer
    fn block_component_footprints(&mut self, board: Board, layout: &mut Layout) {
        for (component_name, _) in &layout.circuit.component_name_to_component_map {
            let footprint = layout.circuit.calc_component_footprint(component_name.to_string());
            for y in footprint.start.y..=footprint.end.y {
                for x in footprint.start.x..=footprint.end.x {
                    self.block(board, Via::new(x, y));
                }
            }
        }
    }

    fn block_route(&mut self, board: Board, route_step_vec: Vec<LayerVia>) {
        for c in route_step_vec {
            if c.is_wire_layer {
                self.block(board, c.via);
            }
        }
    }

    fn block(&mut self, board: Board, via: Via) {
        let i = board.idx(via);
        self.via_trace_vec[i].is_wire_side_blocked = true;
    }

    fn is_blocked(&self, board: Board, via: Via) -> bool {
        self.via_trace_vec[board.idx(via)].is_wire_side_blocked
    }

    //
    // Nets
    //

    fn join_all_connections(&mut self, board: Board, layout: &mut Layout, nets: &mut Nets) {
        for c in layout.circuit.gen_connection_via_vec() {
            nets.connect(board, layout, c.start, c.end);
        }
    }

    fn register_active_component_pins(&mut self, layout: &mut Layout) {
        for (component_name, component) in &layout.circuit.component_name_to_component_map {
            let pin_via_vec = layout.circuit.calc_component_pins(component_name);
            for (pin_idx, via) in pin_via_vec.iter().enumerate() {
                if !component.dont_care_pin_idx_set.contains(&(pin_idx)) {
                    self.all_pin_set.insert(*via);
                }
            }
        }
    }

    fn add_wire_jumps(&mut self, board: Board, route_section_vec: Vec<LayerStartEndVia>) {
        for section in route_section_vec {
            let start = section.start;
            let end = section.end;
            assert_eq!(start.is_wire_layer, end.is_wire_layer);
            if start.is_wire_layer {
                *self.wire_to_via_ref(board, start.via) = ValidVia::from_via(end.via);
                *self.wire_to_via_ref(board, end.via) = ValidVia::from_via(start.via);
            }
        }
    }
}
