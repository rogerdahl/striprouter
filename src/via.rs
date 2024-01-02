// #![allow(unused)]

use nalgebra::Vector2;
// use std::collections::hash_map::DefaultHasher;
use std::cmp::{Ordering, Reverse};
use std::fmt;
use std::hash::{Hash, Hasher};

// Hold floating point screen and board positions
pub type Pos = Vector2<f32>;

// Hold integer positions
// pub type IntPos = Vector2<usize>;

//
// Via (a hole in the stripboard)
//

pub type Via = Vector2<usize>;

pub fn via_to_str(v: &Via) -> String {
    format!("Via({:02x},{:02x})", v.x, v.y)
}

pub fn via_from_offset(v: &Via, offset: &OffsetVia) -> Via {
    Via::new(
        (v.x as isize + offset.x) as usize,
        (v.y as isize + offset.y) as usize,
    )
}

pub type OffsetVia = Vector2<isize>;

//
// ValidVia
//

#[derive(Eq, PartialEq, PartialOrd, Clone, Debug)]
pub struct ValidVia {
    pub via: Via,
    pub is_valid: bool,
}

impl ValidVia {
    pub fn new() -> Self {
        Self {
            via: Via::new(usize::MAX, usize::MAX),
            is_valid: false,
        }
    }

    pub fn from_via(via: Via) -> Self {
        Self {
            via,
            is_valid: true,
        }
    }
}

//
// LayerVia
//

#[derive(Eq, PartialEq, PartialOrd, Clone, Copy, Hash)]
pub struct LayerVia {
    pub via: Via,
    pub is_wire_layer: bool,
}

impl LayerVia {
    pub fn from_layer_cost_via(p0: &Reverse<LayerCostVia>) -> LayerVia {
        LayerVia::from_via(p0.0.layer_via.via, p0.0.layer_via.is_wire_layer)
    }
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

    // pub fn str(&self) -> String {
    //     format!(
    //         "LayerVia({},{},{})",
    //         self.via.x,
    //         self.via.y,
    //         if self.is_wire_layer { "wire" } else { "strip" }
    //     )
    // }

    pub fn is_target(&self, target_via: Via) -> bool {
        if self.is_wire_layer {
            false
        } else {
            self.is_target_pin(target_via)
        }
    }

    fn is_target_pin(&self, target_via: Via) -> bool {
        self.via == target_via
    }
}

impl fmt::Debug for LayerVia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LayerVia({:02x},{:02x},{})",
            self.via.x,
            self.via.y,
            if self.is_wire_layer { "wire" } else { "strip" }
        )
    }
}

impl fmt::Display for LayerVia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LayerVia({:02x},{:02x},{})",
            self.via.x,
            self.via.y,
            if self.is_wire_layer { "wire" } else { "strip" }
        )
    }
}

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

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct LayerCostVia {
    pub layer_via: LayerVia,
    pub cost: usize,
}

impl PartialOrd for LayerCostVia {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_tuple = (
            self.cost,
            self.layer_via.via.x,
            self.layer_via.via.y,
            self.layer_via.is_wire_layer,
        );
        let other_tuple = (
            other.cost,
            other.layer_via.via.x,
            other.layer_via.via.y,
            other.layer_via.is_wire_layer,
        );
        self_tuple.partial_cmp(&other_tuple)
    }
}

impl Ord for LayerCostVia {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_tuple = (
            self.cost,
            self.layer_via.via.x,
            self.layer_via.via.y,
            self.layer_via.is_wire_layer,
        );
        let other_tuple = (
            other.cost,
            other.layer_via.via.x,
            other.layer_via.via.y,
            other.layer_via.is_wire_layer,
        );
        self_tuple.cmp(&other_tuple).reverse()
    }
}

impl LayerCostVia {
    pub fn new() -> Self {
        Self {
            layer_via: LayerVia::new(),
            cost: 0,
        }
    }

    pub fn from_layer_via(layer_via: LayerVia, cost: usize) -> Self {
        Self { layer_via, cost }
    }

    pub fn from_values(x: usize, y: usize, is_wire_layer: bool, cost: usize) -> Self {
        Self {
            layer_via: LayerVia::from_via(Via::new(x, y), is_wire_layer),
            cost,
        }
    }

    pub fn str(&self) -> String {
        format!("{}, {}", self.layer_via.via.x, self.layer_via.via.y)
    }
}

//
// StartEndVia
//

#[derive(Copy, Clone, Debug)]
pub struct StartEndVia {
    pub start: Via,
    pub end: Via,
}

impl StartEndVia {
    pub fn new(start: Via, end: Via) -> Self {
        Self { start, end }
    }
}

//
// LayerStartEndVia
//

#[derive(Eq, PartialEq, PartialOrd, Clone, Debug)]
pub struct LayerStartEndVia {
    pub start: LayerVia,
    pub end: LayerVia,
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

#[derive(Clone, Debug)]
pub struct WireLayerVia {
    pub is_wire_side_blocked: bool,
    pub wire_to_via: ValidVia,
}

impl WireLayerVia {
    pub fn new() -> Self {
        Self {
            is_wire_side_blocked: false,
            wire_to_via: ValidVia::new(),
        }
    }
}

pub type WireLayerViaVec = Vec<WireLayerVia>;

//
// CostVia
//

#[derive(Eq, PartialEq, PartialOrd, Clone, Ord, Debug)]
pub struct CostVia {
    pub wire_cost: usize,
    pub strip_cost: usize,
}

impl CostVia {
    pub fn new() -> Self {
        Self {
            wire_cost: usize::MAX,
            strip_cost: usize::MAX,
        }
    }
}

pub type CostViaVec = Vec<CostVia>;
