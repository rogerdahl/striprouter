use std::sync::Mutex;

const DEFAULT_WIRE_COST: usize = 10;
const DEFAULT_STRIP_COST: usize = 10;
const DEFAULT_VIA_COST: usize = 1;
const DEFAULT_CUT_COST: usize = 100;

pub struct Settings {
    wire_cost: usize,
    strip_cost: usize,
    via_cost: usize,
    pub cut_cost: usize,
    pause: bool,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            wire_cost: DEFAULT_WIRE_COST,
            strip_cost: DEFAULT_STRIP_COST,
            via_cost: DEFAULT_VIA_COST,
            cut_cost: DEFAULT_CUT_COST,
            pause: false,
        }
    }
}

