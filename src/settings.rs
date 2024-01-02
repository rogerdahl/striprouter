// use std::sync::Mutex;

// For testing
const DEFAULT_WIRE_COST: usize = 1;
const DEFAULT_STRIP_COST: usize = 2;
const DEFAULT_VIA_COST: usize = 3;
const DEFAULT_CUT_COST: usize = 4;
// const DEFAULT_WIRE_COST: usize = 10;
// const DEFAULT_STRIP_COST: usize = 10;
// const DEFAULT_VIA_COST: usize = 1;
// const DEFAULT_CUT_COST: usize = 100;

pub struct Settings {
    pub wire_cost: usize,
    pub strip_cost: usize,
    pub via_cost: usize,
    pub cut_cost: usize,
    // pause: bool,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            wire_cost: DEFAULT_WIRE_COST,
            strip_cost: DEFAULT_STRIP_COST,
            via_cost: DEFAULT_VIA_COST,
            cut_cost: DEFAULT_CUT_COST,
            // pause: false,
        }
    }
}

