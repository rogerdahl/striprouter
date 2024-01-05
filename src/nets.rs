use crate::board::Board;
use crate::layout::{Layout, RouteStepVec};
use crate::via::Via;
use std::collections::HashSet;

pub struct Nets {
    pub set_idx_vec: Vec<usize>,
    pub via_set_vec: Vec<HashSet<Via>>,
}

impl Nets {
    pub fn new(board: Board) -> Self {
        Self {
            set_idx_vec: vec![usize::MAX; board.size()],
            via_set_vec: Vec::new(),
        }
    }

    pub fn connect(&mut self, board: Board, layout: &mut Layout, via_a: Via, via_b: Via) {
        let set_idx_a = self.get_via_set_idx(board, layout, via_a);
        let set_idx_b = self.get_via_set_idx(board, layout, via_b);

        // TODO: These asserts get triggered by a fast component drag out of the board on the top
        // side.
        // assert!(set_idx_a < self.via_set_vec.len());
        // assert!(set_idx_b < self.via_set_vec.len());

        if set_idx_a == usize::MAX && set_idx_b == usize::MAX {
            let via_set_idx = self.create_via_set();
            self.via_set_vec[via_set_idx].insert(via_a);
            self.via_set_vec[via_set_idx].insert(via_b);
            self.set_idx_vec[board.idx(via_a)] = via_set_idx;
            self.set_idx_vec[board.idx(via_b)] = via_set_idx;
        } else if set_idx_a != usize::MAX && set_idx_b == usize::MAX {
            let via_set = &mut self.via_set_vec[set_idx_a];
            via_set.insert(via_b);
            self.set_idx_vec[board.idx(via_b)] = set_idx_a;
        } else if set_idx_a == usize::MAX && set_idx_b != usize::MAX {
            let via_set = &mut self.via_set_vec[set_idx_b];
            via_set.insert(via_a);
            self.set_idx_vec[board.idx(via_a)] = set_idx_b;
        } else {
            let via_set_b = self.via_set_vec[set_idx_b].clone();
            let via_set_a = &mut self.via_set_vec[set_idx_a];
            via_set_a.extend(via_set_b);
            for v in &mut self.set_idx_vec {
                if *v == set_idx_b {
                    *v = set_idx_a;
                }
            }
        }
    }

    pub fn connect_route(
        &mut self,
        board: Board,
        layout: &mut Layout,
        route_step_vec: &RouteStepVec,
    ) {
        let mut first = true;
        for c in route_step_vec {
            if first {
                first = false;
                continue;
            }
            if !c.is_wire_layer {
                self.connect(board, layout, route_step_vec[0].via, c.via);
            }
        }
        assert!(self.is_connected(board, layout, route_step_vec[0].via, route_step_vec[1].via));
    }

    pub fn register_pin(&mut self, board: Board, layout: &mut Layout, via: Via) {
        let set_idx = self.get_via_set_idx(board, layout, via);
        if set_idx != usize::MAX {
            let via_set = &mut self.via_set_vec[set_idx];
            via_set.insert(via);
        } else {
            let via_set_idx = self.create_via_set();
            self.via_set_vec[via_set_idx].insert(via);
            self.set_idx_vec[board.idx(via)] = via_set_idx;
        }
    }

    pub fn is_connected(
        &self,
        board: Board,
        layout: &mut Layout,
        current_via: Via,
        target_via: Via,
    ) -> bool {
        let via_set_idx = self.set_idx_vec[board.idx(current_via)];
        if via_set_idx == usize::MAX {
            return false;
        }
        let r = self.via_set_vec[via_set_idx].contains(&target_via);
        r
    }

    pub fn has_connection(&self, board: Board, layout: &mut Layout, via: Via) -> bool {
        let via_set_idx = self.set_idx_vec[board.idx(via)];
        via_set_idx != usize::MAX
    }

    pub fn get_via_set(&self, board: Board, layout: &mut Layout, via: Via) -> &HashSet<Via> {
        let trace_idx = board.idx(via);
        let set_idx = self.set_idx_vec[trace_idx];
        assert_ne!(set_idx, usize::MAX);
        &self.via_set_vec[set_idx]
    }

    fn create_via_set(&mut self) -> usize {
        self.via_set_vec.push(HashSet::new());
        (self.via_set_vec.len() - 1)
    }

    pub fn get_via_set_idx(&self, board: Board, layout: &mut Layout, via: Via) -> usize {
        let trace_idx = board.idx(via);
        self.set_idx_vec[trace_idx]
    }
}
