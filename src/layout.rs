// Please note that the Clone trait is implemented for Layout to allow for creating a
// copy of an instance. Also, the Mutex in Rust is not exactly the same as in C++. It's
// used to achieve interior mutability in Rust. The is_locked method is implemented by
// trying to lock the mutex and checking the result. If the lock is successful, it means
// the mutex was not locked, so it returns false. If the lock fails, it means the mutex
// was already locked, so it returns true.

use crate::board::Board;
use crate::circuit::Circuit;
use crate::settings::Settings;
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Instant;

use crate::via::{CostVia, CostViaVec, LayerStartEndVia, LayerVia, ValidVia, Via, WireLayerViaVec};

pub type RouteStepVec = Vec<LayerVia>;
pub type RouteSectionVec = Vec<LayerStartEndVia>;
pub type RouteVec = Vec<RouteSectionVec>;
pub type StringVec = Vec<String>;
pub type RouteStatusVec = Vec<bool>;
pub type StripCutVec = Vec<Via>;

// Nets
pub type ViaSet = HashSet<Via>;
pub type ViaSetVec = Vec<ViaSet>;
pub type SetIdxVec = Vec<usize>;

pub struct Layout {
    pub circuit: Circuit,
    pub settings: Settings,
    pub board: Board,
    pub cost: usize,
    pub n_completed_routes: usize,
    pub n_failed_routes: usize,
    // pub has_error: bool,
    pub layout_info_vec: StringVec,
    pub route_vec: RouteVec,
    pub strip_cut_vec: StripCutVec,
    pub route_status_vec: RouteStatusVec,
    // Nets
    pub via_set_vec: ViaSetVec,
    pub set_idx_vec: SetIdxVec,
    // Debug
    pub diag_start_via: ValidVia,
    pub diag_end_via: ValidVia,
    pub diag_cost_vec: CostViaVec,
    pub diag_route_step_vec: RouteStepVec,
    pub diag_trace_vec: WireLayerViaVec,
    pub mutex_: Mutex<()>,
    pub timestamp_: Instant,
}

impl Layout {
    pub fn new() -> Self {
        // Initialize the struct here
        Self {
            circuit: Circuit::new(),
            settings: Settings::new(),
            board: Board::new(0, 0),

            cost: 0,
            n_completed_routes: 0,
            n_failed_routes: 0,

            // has_error: false,
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
            mutex_: Mutex::new(()),
            timestamp_: Instant::now(),
        }
    }

    pub fn update_base_timestamp(&mut self) {
        // Update timestamp here
    }

    // Copy Layout
    // pub fn copy_layout(&mut self, other: &mut Layout) {
    //     ///////////////////////////////////////////////////////////////////////////////////////////
    // }

    pub fn copy(&mut self, other: &Self) {
        self.circuit = other.circuit.clone();
        self.settings = other.settings.clone();
        self.board = other.board.clone();

        self.cost = other.cost;
        self.n_completed_routes = other.n_completed_routes;
        self.n_failed_routes = other.n_failed_routes;
        // self.has_error = other.has_error;
        self.layout_info_vec = other.layout_info_vec.clone();
        self.route_vec = other.route_vec.clone();
        self.strip_cut_vec = other.strip_cut_vec.clone();
        self.route_status_vec = other.route_status_vec.clone();
        self.via_set_vec = other.via_set_vec.clone();
        self.set_idx_vec = other.set_idx_vec.clone();
        self.diag_start_via = other.diag_start_via.clone();
        self.diag_end_via = other.diag_end_via.clone();
        self.diag_route_step_vec = other.diag_route_step_vec.clone();
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
}

impl Clone for Layout {
    fn clone(&self) -> Self {
        let mut new_layout = Layout::new();
        new_layout.copy(self);
        new_layout
    }
}
