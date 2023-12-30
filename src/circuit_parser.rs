use std::fs::File;
use std::io;
use std::io::BufRead;
use std::string::String;
use std::vec::Vec;

use lazy_static::lazy_static;
use regex::Regex;

use crate::circuit::{Component, Connection, ConnectionPoint};
use crate::layout::Layout;
use crate::via::Via;

pub struct CircuitFileParser<'a> {
    layout: &'a mut Layout,
    offset: Via,
    aliases: Vec<(String, String)>,
}

lazy_static! {
    static ref ALIAS_RX: Regex = Regex::new(r"^([\w.]+) = ([\w.]+)$").unwrap();
    static ref COMMENT_OR_EMPTY_FULL_RX: Regex = Regex::new(r"^(#.*)?$").unwrap();
    static ref BOARD_SIZE_RX: Regex = Regex::new(r"^board (\d+),(\d+)$").unwrap();
    static ref OFFSET_RX: Regex = Regex::new(r"^offset (-?\d+),(-?\d+)$").unwrap();
    static ref PKG_NAME_RX: Regex = Regex::new(r"^(\w+)\s(.*)").unwrap();
    static ref PKG_SEP_RX: Regex = Regex::new(r"\s+").unwrap();
    static ref PKG_POS_RX: Regex = Regex::new(r"(-?\d+),(-?\d+)").unwrap();
    static ref COMPONENT_FULL_RX: Regex = Regex::new(r"^(\w+) (\w+) ?(\d+),(\d+)$").unwrap();
    static ref CONNECTION_FULL_RX: Regex = Regex::new(r"^(\w+)\.(\d+) (\w+)\.(\d+)$").unwrap();
    static ref DONT_CARE_FULL_RX: Regex = Regex::new(r"^(\w+) (\d+(,|$))+").unwrap();
    static ref DONT_CARE_PIN_IDX_RX: Regex = Regex::new(r"(\d+)(,|$)").unwrap();
}

impl<'a> CircuitFileParser<'a> {
    pub fn new(layout: &'a mut Layout) -> Self {
        Self {
            layout,
            offset: Via::new(0, 0),
            aliases: Vec::new(),
        }
    }

    pub fn parse(&mut self, circuit_file_path: &str) {
        let file = File::open(circuit_file_path).expect("Cannot read .circuit file");
        let reader = io::BufReader::new(file);

        let mut line_idx = 0;
        for line in reader.lines() {
            line_idx += 1;
            let mut line_str = line.unwrap();
            line_str = line_str.split_whitespace().collect::<Vec<&str>>().join(" ");
            line_str = line_str.split(", ").collect::<Vec<&str>>().join(",");
            line_str = line_str.trim().to_string();
            match self.parse_line(line_str.clone()) {
                Ok(_) => (),
                Err(error_str) => {
                    self.layout.circuit.parser_error_vec.push(
                        format!("Error on line {}: {}: {}", line_idx, line_str, error_str)
                    );
                }
            }
        }
        self.layout.is_ready_for_routing = !self.layout.circuit.has_parser_error();
    }


    fn parse_line(&mut self, mut line_str: String) -> Result<(), String> {
        // Substitute aliases
        for alias in &self.aliases {
            let re = Regex::new(&regex::escape(&alias.0)).unwrap();
            let tmp = re.replace_all(&line_str, &alias.1).to_string();
            if line_str != tmp {
                line_str = tmp;
            }
        }

        // Connections are most common, so they are parsed first to improve
        // performance.
        return if self.parse_connection(&line_str) {
            Ok(())
        } else if self.parse_comment_or_empty(&line_str) {
            Ok(())
        } else if self.parse_board(&line_str) {
            Ok(())
        } else if self.parse_offset(&line_str) {
            Ok(())
        } else if self.parse_package(&line_str) {
            Ok(())
        } else if self.parse_component(&line_str) {
            Ok(())
        } else if self.parse_dont_care(&line_str) {
            Ok(())
        } else if self.parse_alias(&line_str) {
            Ok(())
        } else {
            Err("Invalid line".to_string())
        };
    }


    fn parse_alias(&mut self, line_str: &str) -> bool {
        if let Some(captures) = ALIAS_RX.captures(line_str) {
            self.aliases.push((captures[1].to_string(), captures[2].to_string()));
            return true;
        }
        false
    }


    fn parse_comment_or_empty(&self, line_str: &str) -> bool {
        COMMENT_OR_EMPTY_FULL_RX.is_match(line_str)
    }


    fn parse_board(&mut self, line_str: &str) -> bool {
        if let Some(captures) = BOARD_SIZE_RX.captures(line_str) {
            self.layout.grid_w = captures[1].parse::<i32>().unwrap();
            self.layout.grid_h = captures[2].parse::<i32>().unwrap();
            return true;
        }
        false
    }


    fn parse_offset(&mut self, line_str: &str) -> bool {
        if let Some(captures) = OFFSET_RX.captures(line_str) {
            self.offset.x = captures[1].parse::<i32>().unwrap();
            self.offset.y = captures[2].parse::<i32>().unwrap();
            return true;
        }
        false
    }

    fn parse_package(&mut self, line_str: &str) -> bool {
        if let Some(captures) = PKG_NAME_RX.captures(line_str) {
            let pkg_name = captures[1].to_string();
            let pkg_pos = captures[2].to_string();
            let mut v = Vec::new();
            for s in PKG_SEP_RX.split(&pkg_pos) {
                if let Some(captures) = PKG_POS_RX.captures(s) {
                    v.push(Via::new(
                        captures[1].parse::<i32>().unwrap(),
                        captures[2].parse::<i32>().unwrap(),
                    ));
                } else {
                    return false;
                }
            }
            self.layout.circuit.package_to_pos_map.insert(pkg_name, v);
            return true;
        }
        false
    }


    fn parse_component(&mut self, line_str: &str) -> bool {
        if let Some(captures) = COMPONENT_FULL_RX.captures(line_str) {
            let component_name = captures[1].to_string();
            let package_name = captures[2].to_string();
            let x = captures[3].parse::<i32>().unwrap();
            let y = captures[4].parse::<i32>().unwrap();
            if !self.layout.circuit.package_to_pos_map.contains_key(&package_name) {
                panic!("Unknown package: {}", package_name);
            }
            let p = Via::new(x, y) + self.offset;
            let mut i = 0;
            for v in self.layout.circuit.package_to_pos_map.get(&package_name).unwrap() {
                if p.x + v.x < 0 || p.x + v.x >= self.layout.grid_w || p.y + v.y < 0 || p.y + v.y >= self.layout.grid_h {
                    panic!("Component pin outside of board: {}.{}", component_name, i + 1);
                }
                i += 1;
            }
            let component = Component::new(package_name, p);
            self.layout.circuit.component_name_to_component_map.insert(component_name, component);
            return true;
        }
        false
    }


    fn parse_dont_care(&mut self, line_str: &str) -> bool {
        if let Some(captures) = DONT_CARE_FULL_RX.captures(line_str) {
            let component_name = captures[1].to_string();
            let component = self.layout.circuit.component_name_to_component_map.get_mut(&component_name);
            match component {
                None => panic!("Unknown component: {}", component_name),
                Some(component) => {
                    let package_pos_vec = self.layout.circuit.package_to_pos_map.get(&component.package_name).unwrap();
                    for captures in DONT_CARE_PIN_IDX_RX.captures_iter(line_str) {
                        let dont_care_pin_idx = captures[1].parse::<i32>().unwrap();
                        if dont_care_pin_idx < 1 || dont_care_pin_idx as usize > package_pos_vec.len() {
                            panic!(
                                "Invalid \"Don't Care\" pin number for {}: {}. Must be between 1 and {} (including)",
                                component_name, dont_care_pin_idx, package_pos_vec.len()
                            );
                        }
                        component.dont_care_pin_idx_set.insert(dont_care_pin_idx - 1);
                    }
                    return true;
                }
            }
        }
        false
    }

    fn parse_connection(&mut self, line_str: &str) -> bool {
        if let Some(captures) = CONNECTION_FULL_RX.captures(line_str) {
            let start = ConnectionPoint::new(
                captures[1].to_string(),
                captures[2].parse::<i32>().unwrap() - 1,
            );
            let end = ConnectionPoint::new(
                captures[3].to_string(),
                captures[4].parse::<i32>().unwrap() - 1,
            );
            self.check_connection_point(&start);
            self.check_connection_point(&end);
            if start.component_name != end.component_name || start.pin_idx != end.pin_idx {
                self.layout.circuit.connection_vec.push(Connection::new(start, end));
            }
            return true;
        }
        false
    }

    fn check_connection_point(&self, connection_point: &ConnectionPoint) {
        let component = self.layout.circuit.component_name_to_component_map.get(&connection_point.component_name);
        match component {
            None => panic!("Unknown component: {}", connection_point.component_name),
            Some(component) => {
                let package_pos_vec = self.layout.circuit.package_to_pos_map.get(&component.package_name).unwrap();
                let pin_idx_1_base = connection_point.pin_idx + 1;
                if pin_idx_1_base < 1 || pin_idx_1_base as usize > package_pos_vec.len() {
                    panic!(
                        "Invalid pin number for {}.{}. Must be between 1 and {} (including)",
                        connection_point.component_name, pin_idx_1_base, package_pos_vec.len()
                    );
                }
                if component.dont_care_pin_idx_set.contains(&connection_point.pin_idx) {
                    panic!(
                        "Invalid pin number for {}.{}. Pin has been set as \"Don't Care\"",
                        connection_point.component_name, pin_idx_1_base
                    );
                }
            }
        }
    }
}

