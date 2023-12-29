use std::collections::{HashMap, HashSet};
use std::i32;

use crate::via::{Via, StartEndVia};

// Packages

type PackageRelPosVec = Vec<Via>;
type PackageToPosMap = HashMap<String, PackageRelPosVec>;

// Components

type DontCarePinIdxSet = HashSet<i32>;

pub struct Component {
    pub(crate) package_name: String,
    pin0_abs_pos: Via,
    pub(crate) dont_care_pin_idx_set: DontCarePinIdxSet,
}

impl Component {
    // Apparently, when the parameters have the same names as the struct fields,
    // the struct fields can be initialized with the parameters without having
    // to specify the field names.
    pub fn new(package_name: String, pin0_abs_pos: Via) -> Self {
        Self {
            package_name,
            pin0_abs_pos,
            dont_care_pin_idx_set: DontCarePinIdxSet::new(),
        }
    }
}

type ComponentNameToComponentMap = HashMap<String, Component>;

// Connections

pub struct ConnectionPoint {
    pub(crate) component_name: String,
    pub(crate) pin_idx: i32,
}

impl ConnectionPoint {
    pub fn new(component_name: String, pin_idx: i32) -> Self {
        Self {
            component_name,
            pin_idx,
        }
    }
}

pub struct Connection {
    start: ConnectionPoint,
    end: ConnectionPoint,
}

impl Connection {
    pub fn new(start: ConnectionPoint, end: ConnectionPoint) -> Self {
        Self { start, end }
    }
}

// Circuit

type ConnectionVec = Vec<Connection>;
type ConnectionViaVec = Vec<StartEndVia>;
type StringVec = Vec<String>;
type PinViaVec = Vec<Via>;

pub struct Circuit {
    pub(crate) package_to_pos_map: PackageToPosMap,
    pub(crate) component_name_to_component_map: ComponentNameToComponentMap,
    pub(crate) connection_vec: ConnectionVec,
    pub(crate) parser_error_vec: StringVec,
}

impl Circuit {
    pub fn new() -> Self {
        Self {
            package_to_pos_map: PackageToPosMap::new(),
            component_name_to_component_map: ComponentNameToComponentMap::new(),
            connection_vec: ConnectionVec::new(),
            parser_error_vec: StringVec::new(),
        }
    }

    pub fn has_parser_error(&self) -> bool {
        !self.parser_error_vec.is_empty()
    }

    pub fn gen_connection_via_vec(&self) -> ConnectionViaVec {
        let mut v = Vec::new();
        for c in &self.connection_vec {
            let start_component = self.component_name_to_component_map.get(&c.start.component_name).unwrap();
            let end_component = self.component_name_to_component_map.get(&c.end.component_name).unwrap();

            let start_rel_pin = self.package_to_pos_map.get(&start_component.package_name).unwrap()[c.start.pin_idx as usize];
            let end_rel_pin = self.package_to_pos_map.get(&end_component.package_name).unwrap()[c.end.pin_idx as usize];

            let start_abs_pin = start_rel_pin + start_component.pin0_abs_pos;
            let end_abs_pin = end_rel_pin + end_component.pin0_abs_pos;

            v.push(StartEndVia::new(start_abs_pin, end_abs_pin));
        }
        v
    }

    pub fn calc_component_footprint(&self, component_name: String) -> StartEndVia {
        let mut v = StartEndVia::new(Via::new(i32::MAX, i32::MAX), Via::new(0, 0));
        let component = self.component_name_to_component_map.get(&component_name).unwrap();
        for cc in self.package_to_pos_map.get(&component.package_name).unwrap() {
            let c = cc + component.pin0_abs_pos;
            if c.x < v.start.x {
                v.start.x = c.x;
            }
            if c.x > v.end.x {
                v.end.x = c.x;
            }
            if c.y < v.start.y {
                v.start.y = c.y;
            }
            if c.y > v.end.y {
                v.end.y = c.y;
            }
        }
        v
    }

    pub fn calc_component_pins(&self, component_name: String) -> PinViaVec {
        let mut v = PinViaVec::new();
        let component = self.component_name_to_component_map.get(&component_name).unwrap();
        for cc in self.package_to_pos_map.get(&component.package_name).unwrap() {
            let c = cc + component.pin0_abs_pos;
            v.push(c);
        }
        v
    }
}

