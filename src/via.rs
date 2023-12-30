// #![allow(unused)]

use nalgebra::{Vector2};
// use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

// Hold floating point screen and board positions
pub(crate) type Pos = Vector2<f32>;

// Hold integer positions
pub(crate) type IntPos = Vector2<i32>;

//
// Via (a hole in the stripboard)
//

pub(crate) type Via = Vector2<i32>;

fn str(v: &Via) -> String {
    format!("{},{}", v.x, v.y)
}

// impl Hash for Via {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.x.hash(state);
//         self.y.hash(state);
//     }
// }

// impl PartialEq for Via {
//     fn eq(&self, other: &Self) -> bool {
//         self.x == other.x && self.y == other.y
//     }
// }

// impl Eq for Via {}

// impl PartialOrd for Via {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl Ord for Via {
//     fn cmp(&self, other: &Self) -> Ordering {
//         match self.x.cmp(&other.x) {
//             Ordering::Equal => self.y.cmp(&other.y),
//             other => other,
//         }
//     }
// }

//
// ValidVia
//

#[derive(Eq, PartialEq, PartialOrd, Clone)]
pub struct ValidVia {
    pub(crate) via: Via,
    pub(crate) is_valid: bool,
}

impl ValidVia {
    pub fn new() -> Self {
        Self {
            via: Via::new(0, 0),
            is_valid: false,
        }
    }

    pub fn from_via(via: Via) -> Self {
        Self {
            via,
            is_valid: true,
        }
    }

    pub fn from_via_valid(via: Via, is_valid: bool) -> Self {
        Self { via, is_valid }
    }
}

//
// LayerVia
//

#[derive(Eq, PartialEq, PartialOrd, Clone)]
pub struct LayerVia {
    pub(crate) via: Via,
    pub(crate) is_wire_layer: bool,
}

impl LayerVia {
    pub fn new() -> Self {
        Self {
            via: Via::new(0, 0),
            is_wire_layer: false,
        }
    }

    pub fn from_via(via: Via, is_wire_layer: bool) -> Self {
        Self { via, is_wire_layer }
    }

    pub fn str(&self) -> String {
        format!("{}, {}", self.via.x, self.via.y)
    }
}

// impl Hash for LayerVia {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.via.hash(state);
//         self.is_wire_layer.hash(state);
//     }
// }
//
// impl PartialEq for LayerVia {
//     fn eq(&self, other: &Self) -> bool {
//         self.via == other.via && self.is_wire_layer == other.is_wire_layer
//     }
// }
//
// impl Eq for LayerVia {}
//
// impl PartialOrd for LayerVia {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }
//
// // Implement Ord for LayerVia:
// // - sort by via
// // - if via is equal, sort by is_wire_layer
// impl Ord for LayerVia {
//     fn cmp(&self, other: &Self) -> Ordering {
//         match self.via.cmp(&other.via) {
//             Ordering::Equal => self.is_wire_layer.cmp(&other.is_wire_layer),
//             other => other,
//         }
//     }
// }

//
// LayerCostVia
//

// #[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct LayerCostVia {
    layer_via: LayerVia,
    cost: i32,
}

impl LayerCostVia {
    pub fn new() -> Self {
        Self {
            layer_via: LayerVia::new(),
            cost: 0,
        }
    }

    pub fn from_layer_via(layer_via: LayerVia, cost: i32) -> Self {
        Self { layer_via, cost }
    }

    pub fn from_values(x: i32, y: i32, is_wire_layer: bool, cost: i32) -> Self {
        Self {
            layer_via: LayerVia::from_via(Via::new(x, y), is_wire_layer),
            cost,
        }
    }

    pub fn str(&self) -> String {
        format!("{}, {}", self.layer_via.via.x, self.layer_via.via.y)
    }
}

// impl PartialEq for LayerCostVia {
//     fn eq(&self, other: &Self) -> bool {
//         self.layer_via == other.layer_via
//     }
// }

// impl PartialEq<Self> for LayerCostVia {
//     fn eq(&self, other: &Self) -> bool {
//         self.layer_via == other.layer_via
//     }
// }

// impl PartialEq<Self> for LayerCostVia {
//     fn eq(&self, other: &Self) -> bool {
//         self.layer_via == other.layer_via
//     }
// }

// impl PartialOrd for LayerCostVia {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(&other))
//     }
// }

// impl Ord for LayerCostVia {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.layer_via.cmp(&other.layer_via)
//     }
// }

//
// StartEndVia
//

pub struct StartEndVia {
    pub(crate) start: Via,
    pub(crate) end: Via,
}

impl StartEndVia {
    pub fn new(start: Via, end: Via) -> Self {
        Self { start, end }
    }
}

//
// LayerStartEndVia
//

#[derive(Eq, PartialEq, PartialOrd, Clone)]
pub struct LayerStartEndVia {
    pub(crate) start: LayerVia,
    pub(crate) end: LayerVia,
}

impl LayerStartEndVia {
    pub fn new() -> Self {
        Self {
            start: LayerVia::new(),
            end: LayerVia::new(),
        }
    }

    pub fn from_layer_vias(start: LayerVia, end: LayerVia) -> Self {
        Self { start, end }
    }
}

//
// WireLayerVia
//

pub struct WireLayerVia {
    pub(crate) is_wire_side_blocked: bool,
    pub(crate) wire_to_via: ValidVia,
}

impl WireLayerVia {
    pub fn new() -> Self {
        Self {
            is_wire_side_blocked: false,
            wire_to_via: ValidVia::new(),
        }
    }
}

pub(crate) type WireLayerViaVec = Vec<WireLayerVia>;

//
// CostVia
//

pub struct CostVia {
    pub(crate) wire_cost: i32,
    pub(crate) strip_cost: i32,
}

impl CostVia {
    pub fn new() -> Self {
        Self {
            wire_cost: i32::MAX,
            strip_cost: i32::MAX,
        }
    }
}

pub(crate) type CostViaVec = Vec<CostVia>;
