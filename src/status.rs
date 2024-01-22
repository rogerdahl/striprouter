// use lazy_static::lazy_static;
use std::sync::Mutex;

// lazy_static! {
//     pub static ref STATUS_MUTEX: Mutex<()> = Mutex::new(());
// }

pub struct Status {
    pub ms_per_frame: f32,
    pub checked_total: usize,
    pub checked_per_second: f32,

    pub wire_cost: i32,
    pub strip_cost: i32,
    pub via_cost: i32,
    pub cut_cost: i32,

    pub zoom: f32,

    pub current_layout_completed_routes: usize,
    pub current_layout_failed_routes: usize,
    pub current_layout_cost: usize,

    pub best_layout_completed_routes: usize,
    pub best_layout_failed_routes: usize,
    pub best_layout_cost: usize,

    pub show_rats_nest: bool,
    pub show_only_failed: bool,
    pub show_current_layout: bool,
    pub pause_router: bool,
}

impl Status {
    pub fn new() -> Self {
        Self {
            ms_per_frame: 0.0,
            checked_total: 0,
            checked_per_second: 0.0,
            wire_cost: 10,
            strip_cost: 10,
            via_cost: 1,
            cut_cost: 100,
            zoom: 15.0,
            current_layout_completed_routes: 0,
            current_layout_failed_routes: 0,
            current_layout_cost: 0,
            best_layout_completed_routes: 0,
            best_layout_failed_routes: 0,
            best_layout_cost: 0,
            show_rats_nest: false,
            show_only_failed: false,
            show_current_layout: false,
            pause_router: false,
        }
    }
}
