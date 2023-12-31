use crate::layout::Layout;
use crate::nets::Nets;
use crate::thread_stop::ThreadStop;
use crate::via::{Via, WireLayerVia};
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Duration;

pub struct Router<'a, 'b> {
    layout: &'a mut Layout,
    connection_idx_vec: &'a Vec<i32>,
    input_layout: &'a Layout,
    current_layout: &'a Layout,
    nets: &'b Nets<'a>,
    thread_stop: &'a ThreadStop,
    max_render_delay: Duration,
    all_pin_set: HashSet<Via>,
    via_trace_vec: Vec<WireLayerVia>,
}

impl<'a, 'b> Router<'a, 'b> {
    pub fn new(
        layout: &'a mut Layout,
        connection_idx_vec: &'a Vec<i32>,
        thread_stop: &'a ThreadStop,
        input_layout: &'a Layout,
        current_layout: &'a Layout,
        max_render_delay: Duration,
    ) -> Self {
        Self {
            layout,
            connection_idx_vec,
            input_layout,
            current_layout,
            nets: &Nets::new(layout),
            thread_stop,
            all_pin_set: HashSet::new(),
            max_render_delay,
            via_trace_vec: vec![WireLayerVia::default(); (layout.grid_w * layout.grid_h) as usize],
        }
    }

    pub fn route(&mut self) -> bool {
        self.block_component_footprints();
        self.join_all_connections();
        self.register_active_component_pins();
        let is_aborted = self.route_all();
        self.layout.strip_cut_vec = self.find_strip_cuts();
        self.layout.cost += self.layout.settings.cut_cost * self.layout.strip_cut_vec.len() as i32;
        self.layout.is_ready_for_eval = true;
        if self.layout.has_error {
            self.layout.diag_trace_vec = self.via_trace_vec.clone();
        }
        is_aborted
    }

    // Other methods...
}
