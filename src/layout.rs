// Please note that the Clone trait is implemented for Layout to allow for creating a
// copy of an instance. Also, the Mutex in Rust is not exactly the same as in C++. It's
// used to achieve interior mutability in Rust. The is_locked method is implemented by
// trying to lock the mutex and checking the result. If the lock is successful, it means
// the mutex was not locked, so it returns false. If the lock fails, it means the mutex
// was already locked, so it returns true.

use std::sync::Mutex;
use std::time::Instant;
use std::collections::HashSet;
use crate::circuit::Circuit;
use crate::settings::Settings;

use crate::via::{CostVia, CostViaVec, WireLayerViaVec, LayerStartEndVia, LayerVia, ValidVia, Via};


pub type RouteStepVec = Vec<LayerVia>;
pub type RouteSectionVec = Vec<LayerStartEndVia>;
pub type RouteVec = Vec<RouteSectionVec>;
pub type StringVec = Vec<String>;
pub type RouteStatusVec = Vec<bool>;
pub type StripCutVec = Vec<Via>;


// Nets
pub type ViaSet = HashSet<Via>;
pub type ViaSetVec = Vec<ViaSet>;
pub type SetIdxVec = Vec<i32>;

pub struct Layout {
    pub(crate) circuit: Circuit,
    pub(crate) settings: Settings,
    pub(crate) grid_w: i32,
    pub(crate) grid_h: i32,
    pub(crate) cost: i64,
    pub(crate) n_completed_routes: i32,
    pub(crate) n_failed_routes: i32,
    pub(crate) num_shortcuts: i32,
    pub(crate) is_ready_for_routing: bool,
    pub(crate) is_ready_for_eval: bool,
    pub(crate) has_error: bool,
    pub(crate) layout_info_vec: StringVec,
    pub(crate) route_vec: RouteVec,
    pub(crate) strip_cut_vec: StripCutVec,
    pub(crate) route_status_vec: RouteStatusVec,
    // Nets
    pub(crate) via_set_vec: ViaSetVec,
    pub(crate) set_idx_vec: SetIdxVec,
    // Debug
    pub(crate) diag_start_via: ValidVia,
    pub(crate) diag_end_via: ValidVia,
    pub(crate) diag_cost_vec: CostViaVec,
    pub(crate) diag_route_step_vec: RouteStepVec,
    pub(crate) diag_trace_vec: WireLayerViaVec,
    pub(crate) error_string_vec: StringVec,
    pub(crate) mutex_: Mutex<()>,
    pub(crate) timestamp_: Instant,
}

impl Layout {
    pub fn new() -> Self {
        // Initialize the struct here
        Self {
            circuit: Circuit::new(),
            settings: Settings::new(),
            grid_w: 0,
            grid_h: 0,

            cost: 0,
            n_completed_routes: 0,
            n_failed_routes: 0,
            num_shortcuts: 0,

            is_ready_for_routing: false,
            is_ready_for_eval: false,
            has_error: false,

            layout_info_vec: StringVec::new(),
            route_vec: RouteVec::new(),
            strip_cut_vec: StripCutVec::new(),
            route_status_vec: RouteStatusVec::new(),

            // Nets
            via_set_vec: ViaSetVec::new(),
            set_idx_vec: SetIdxVec::new(),

            // Debug
            diag_start_via: ValidVia::new(),
            diag_end_via: ValidVia::new(),

            diag_cost_vec: CostViaVec::new(),
            diag_route_step_vec: RouteStepVec::new(),
            diag_trace_vec: WireLayerViaVec::new(),
            error_string_vec: StringVec::new(),
            mutex_: Mutex::new(()),
            timestamp_: Instant::now(),
        }
    }

    pub fn update_base_timestamp(&mut self) {
        // Update timestamp here
    }

    // Add other methods here

    // Copy Layout
    pub fn copy_layout(&mut self, other: &mut Layout) {

    }

    pub fn copy(&mut self, other: &Self) {
        self.grid_w = other.grid_w;
        self.grid_h = other.grid_h;
        self.cost = other.cost;
        self.n_completed_routes = other.n_completed_routes;
        self.n_failed_routes = other.n_failed_routes;
        self.num_shortcuts = other.num_shortcuts;
        self.is_ready_for_routing = other.is_ready_for_routing;
        self.is_ready_for_eval = other.is_ready_for_eval;
        self.has_error = other.has_error;
        self.layout_info_vec = other.layout_info_vec.clone();
        self.route_vec = other.route_vec.clone();
        self.strip_cut_vec = other.strip_cut_vec.clone();
        self.route_status_vec = other.route_status_vec.clone();
        self.via_set_vec = other.via_set_vec.clone();
        self.set_idx_vec = other.set_idx_vec.clone();
        self.diag_start_via = other.diag_start_via.clone();
        self.diag_end_via = other.diag_end_via.clone();
        self.diag_route_step_vec = other.diag_route_step_vec.clone();
        self.error_string_vec = other.error_string_vec.clone();
        self.timestamp_ = other.timestamp_;
    }

    pub fn is_based_on(&self, other: &Self) -> bool {
        self.timestamp_ == other.timestamp_
    }

    pub fn get_base_timestamp(&self) -> &Instant {
        &self.timestamp_
    }

    pub fn scope_lock(&self) -> std::sync::LockResult<std::sync::MutexGuard<'_, ()>> {
        self.mutex_.lock()
    }

    pub fn thread_safe_copy(&self) -> Self {
        let _lock = self.scope_lock().unwrap();
        let copy = self.clone();
        copy
    }

    pub fn is_locked(&self) -> bool {
        let result = self.mutex_.try_lock();
        match result {
            Ok(_) => false,
            Err(_) => true,
        }
    }

    pub fn idx(&self, v: &Via) -> i32 {
        v.x + self.grid_w * v.y
    }
}

impl Clone for Layout {
    fn clone(&self) -> Self {
        let mut new_layout = Layout::new();
        new_layout.copy(self);
        new_layout
    }
}
