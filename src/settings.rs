use std::sync::Mutex;

const DEFAULT_WIRE_COST: i32 = 10;
const DEFAULT_STRIP_COST: i32 = 10;
const DEFAULT_VIA_COST: i32 = 1;
const DEFAULT_CUT_COST: i32 = 100;

pub struct Settings {
    wire_cost: i32,
    strip_cost: i32,
    via_cost: i32,
    cut_cost: i32,
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

