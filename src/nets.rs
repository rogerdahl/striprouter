use std::collections::HashSet;
use crate::layout::{Layout, RouteStepVec};
use crate::via::Via;

pub struct Nets<'a> {
    layout: &'a mut Layout,
    set_idx_vec: Vec<i32>,
    via_set_vec: Vec<HashSet<Via>>,
}

impl<'a> Nets<'a> {
    pub fn new(layout: &'a mut Layout) -> Self {
        Self {
            layout,
            set_idx_vec: vec![-1; (layout.grid_w * layout.grid_h) as usize],
            via_set_vec: Vec::new(),
        }
    }

    pub fn connect(&mut self, via_a: Via, via_b: Via) {
        let set_idx_a = self.get_via_set_idx(via_a);
        let set_idx_b = self.get_via_set_idx(via_b);

        // TODO: These asserts get triggered by a fast component drag out of the board on the top
        // side.
        assert!(set_idx_a < self.via_set_vec.len() as i32);
        assert!(set_idx_b < self.via_set_vec.len() as i32);

        if set_idx_a == -1 && set_idx_b == -1 {
            let via_set_idx = self.create_via_set();
            self.via_set_vec[via_set_idx as usize].insert(via_a);
            self.via_set_vec[via_set_idx as usize].insert(via_b);
            self.set_idx_vec[self.layout.idx(&via_a) as usize] = via_set_idx;
            self.set_idx_vec[self.layout.idx(&via_b) as usize] = via_set_idx;
        } else if set_idx_a != -1 && set_idx_b == -1 {
            let via_set = &mut self.via_set_vec[set_idx_a as usize];
            via_set.insert(via_b);
            self.set_idx_vec[self.layout.idx(&via_b) as usize] = set_idx_a;
        } else if set_idx_a == -1 && set_idx_b != -1 {
            let via_set = &mut self.via_set_vec[set_idx_b as usize];
            via_set.insert(via_a);
            self.set_idx_vec[self.layout.idx(&via_a) as usize] = set_idx_b;
        } else {
            let via_set_a = &mut self.via_set_vec[set_idx_a as usize];
            let via_set_b = &self.via_set_vec[set_idx_b as usize];
            via_set_a.extend(via_set_b);
            for v in &mut self.set_idx_vec {
                if *v == set_idx_b {
                    *v = set_idx_a;
                }
            }
        }
    }

    pub fn connect_route(&mut self, route_step_vec: &RouteStepVec) {
        let mut first = true;
        for c in route_step_vec {
            if first {
                first = false;
                continue;
            }
            if !c.is_wire_layer {
                self.connect(route_step_vec[0].via, c.via);
            }
        }
        assert!(self.is_connected(route_step_vec[0].via, route_step_vec[1].via));
    }

    pub fn register_pin(&mut self, via: Via) {
        let set_idx = self.get_via_set_idx(via);
        if set_idx != -1 {
            let via_set = &mut self.via_set_vec[set_idx as usize];
            via_set.insert(via);
        } else {
            let via_set_idx = self.create_via_set();
            self.via_set_vec[via_set_idx as usize].insert(via);
            self.set_idx_vec[self.layout.idx(&via) as usize] = via_set_idx;
        }
    }

    pub fn is_connected(&self, current_via: Via, target_via: Via) -> bool {
        let via_set_idx = self.set_idx_vec[self.layout.idx(&current_via) as usize];
        if via_set_idx == -1 {
            return false;
        }
        let r = self.via_set_vec[via_set_idx as usize].contains(&target_via);
        r
    }

    pub fn has_connection(&self, via: Via) -> bool {
        let via_set_idx = self.set_idx_vec[self.layout.idx(&via) as usize];
        via_set_idx != -1
    }

    pub fn get_via_set(&self, via: Via) -> &HashSet<Via> {
        let trace_idx = self.layout.idx(&via) as usize;
        let set_idx = self.set_idx_vec[trace_idx];
        assert_ne!(set_idx, -1);
        &self.via_set_vec[set_idx as usize]
    }

    fn create_via_set(&mut self) -> i32 {
        self.via_set_vec.push(HashSet::new());
        (self.via_set_vec.len() - 1) as i32
    }

    pub fn get_via_set_idx(&self, via: Via) -> i32 {
        let trace_idx = self.layout.idx(&via) as usize;
        self.set_idx_vec[trace_idx]
    }

}