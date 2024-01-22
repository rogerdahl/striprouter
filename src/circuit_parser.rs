use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::string::String;
use std::vec::Vec;

use lazy_static::lazy_static;
use regex::Regex;

use crate::circuit::{Component, Connection, ConnectionPoint};
use crate::layout::Layout;
use crate::via::{via_add_offset, via_from_offset, OffsetVia, Via};

pub struct CircuitFileParser<'a> {
    layout: &'a mut Layout,
    offset: OffsetVia,
    aliases: Vec<(String, String)>,
}

lazy_static! {
    static ref WHITESPACE_SEP_RX: Regex = Regex::new(r"\s+").unwrap();
    // static ref ALIAS_RX: Regex = Regex::new(r"^([\w.]+) = ([\w.]+)$").unwrap();
    static ref COMMENT_OR_EMPTY_FULL_RX: Regex = Regex::new(r"^(#.*)?$").unwrap();
    static ref BOARD_SIZE_RX: Regex = Regex::new(r"^board (\d+),(\d+)$").unwrap();
    static ref OFFSET_RX: Regex = Regex::new(r"^offset (-?\d+),(-?\d+)$").unwrap();
    static ref PKG_NAME_RX: Regex = Regex::new(r"^(\w+)\s(.*)").unwrap();
    static ref PKG_POS_RX: Regex = Regex::new(r"(-?\d+),(-?\d+)").unwrap();
    static ref COMPONENT_FULL_RX: Regex = Regex::new(r"^(\w+) (\w+) ?(\d+),(\d+)$").unwrap();
    static ref CONNECTION_FULL_RX: Regex = Regex::new(r"^(\w+)\.(\d+) (\w+)\.(\d+)$").unwrap();
    static ref DONT_CARE_FULL_RX: Regex = Regex::new(r"^(\w+) ((\d+( |$))+)$").unwrap();
}

impl<'a> CircuitFileParser<'a> {
    pub fn new(layout: &'a mut Layout) -> Self {
        Self {
            layout,
            offset: OffsetVia::new(0, 0),
            aliases: Vec::new(),
        }
    }

    pub fn parse(&mut self, circuit_file_path: &OsStr) {
        println!("{:?}", circuit_file_path.to_str());
        let file = File::open(circuit_file_path).expect("Cannot read .circuit file");
        let reader = io::BufReader::new(file);

        let mut line_idx = 0;
        for line in reader.lines() {
            line_idx += 1;
            let mut line = line.unwrap();
            line = line.split_whitespace().collect::<Vec<&str>>().join(" ");
            // line = line.split(", ").collect::<Vec<&str>>().join(",");
            line = line.trim().to_string();
            match self.parse_line(line.clone()) {
                Ok(_) => (),
                Err(error) => {
                    self.layout
                        .circuit
                        .parser_error_vec
                        .push(format!("Error on line {}: {}: {}", line_idx, line, error));
                }
            }
        }
    }

    fn parse_line(&mut self, mut line: String) -> Result<(), String> {
        // Substitute aliases
        // TODO: Refactor to record aliases in a map, and substitute them in one pass.
        // for alias in &self.aliases {
        //     let re = Regex::new(&regex::escape(&alias.0)).unwrap();
        //     let tmp = re.replace_all(&line, &alias.1).to_string();
        //     if line != tmp {
        //         line = tmp;
        //     }
        // }

        // We pass the line to line parsers in turn, until one of them succeeds.
        // A line parser returns:
        // - Ok(true): The line was recognized and successfully parsed. The line is
        // considered to be consumed, and we move on to the next line.
        // - Ok(false): The line was not recognized by the parser. The line is still
        // unparsed, and we try parsing it again by sending it to the next parser.
        // - Err(error): The line was recognized, but parsing failed. We record the
        // error and move on to the next line. At this point, parsing of the full
        // circuit file has failed, but we still try to parse the rest of the file to
        // collect as many errors as possible.

        // Connections are most common, so they are parsed first to improve
        // performance.

        println!("{}", line);

        return if self.parse_connection(&line)? {
            Ok(())
        } else if self.parse_comment_or_empty(&line)? {
            Ok(())
        } else if self.parse_board(&line)? {
            Ok(())
        } else if self.parse_offset(&line)? {
            Ok(())
        } else if self.parse_package(&line)? {
            Ok(())
        } else if self.parse_component(&line)? {
            Ok(())
        } else if self.parse_dont_care(&line)? {
            Ok(())
        // } else if self.parse_alias(&line)? {
        //     Ok(())
        } else {
            Err("Unrecognized line".to_string())
        };
    }

    // Alias
    // fn parse_alias(&mut self, line: &str) -> Result<bool, String> {
    //     if let Some(captures) = ALIAS_RX.captures(line) {
    //         self.aliases.push((captures[1].to_string(), captures[2].to_string()));
    //         return Ok(true);
    //     }
    //     Ok(false)
    // }

    // Comment or empty line
    fn parse_comment_or_empty(&self, line: &str) -> Result<bool, String> {
        match COMMENT_OR_EMPTY_FULL_RX.captures(line) {
            Some(_) => return Ok(true),
            None => return Ok(false),
        }
    }

    // Board params (currently just size)
    // board <number of horizontal vias>,<number of vertical vias>
    fn parse_board(&mut self, line: &str) -> Result<bool, String> {
        match BOARD_SIZE_RX.captures(line) {
            Some(captures) => {
                self.layout.board.w = captures[1].parse::<isize>().unwrap() as usize;
                self.layout.board.h = captures[2].parse::<isize>().unwrap() as usize;
                return Ok(true);
            }
            None => return Ok(false),
        }
    }

    // Component position offset. Can be used multiple times to adjust section of
    // circuit. Adds the given offset to the positions of components defined below
    // in the .circuit file. To disable, set to 0,0.
    // offset <relative x pos>, <relative y pos>
    fn parse_offset(&mut self, line: &str) -> Result<bool, String> {
        match OFFSET_RX.captures(line) {
            Some(captures) => {
                self.offset.x = captures[1].parse::<isize>().unwrap();
                self.offset.y = captures[2].parse::<isize>().unwrap();
                return Ok(true);
            }
            None => return Ok(false),
        }
    }

    // Package
    // dip8 0,0 1,0 2,0 3,0 4,0 5,0 6,0 7,0 7,-2 6,-2 5,-2 4,-2 3,-2 2,-2 1,-2
    fn parse_package(&mut self, line: &str) -> Result<bool, String> {
        match PKG_NAME_RX.captures(line) {
            Some(captures) => {
                let pkg_name = captures[1].to_string();
                let pkg_pos = captures[2].to_string();
                let mut v = Vec::new();
                for s in WHITESPACE_SEP_RX.split(&pkg_pos) {
                    if let Some(captures) = PKG_POS_RX.captures(s) {
                        v.push(OffsetVia::new(
                            captures[1].parse::<isize>().unwrap(),
                            captures[2].parse::<isize>().unwrap(),
                        ));
                    } else {
                        return Ok(false);
                    }
                }
                self.layout.circuit.package_to_pos_map.insert(pkg_name, v);
                return Ok(true);
            }
            None => return Ok(false),
        }
    }

    // Component
    // <component name> <package name> <absolute position of component pin 1>
    fn parse_component(&mut self, line: &str) -> Result<bool, String> {
        match COMPONENT_FULL_RX.captures(line) {
            Some(captures) => {
                let component_name = captures[1].to_string();
                let package_name = captures[2].to_string();
                let x = captures[3].parse::<isize>().unwrap();
                let y = captures[4].parse::<isize>().unwrap();
                if !self.layout.circuit.package_to_pos_map.contains_key(&package_name) {
                    return Err(format!("Unknown package: {}", package_name));
                }
                let p = OffsetVia::new(x + self.offset.x, y + self.offset.y);
                let mut i = 0;
                for o in self.layout.circuit.package_to_pos_map.get(&package_name).unwrap() {
                    if p.x + o.x < 0
                        || p.x + o.x >= self.layout.board.w as isize
                        || p.y + o.y < 0
                        || p.y + o.y >= self.layout.board.h as isize
                    {
                        return Err(format!("Component pin outside of board: {}.{}", component_name, i + 1));
                    }
                    i += 1;
                }
                let component = Component::new(package_name, via_from_offset(&p));
                self.layout
                    .circuit
                    .component_name_to_component_map
                    .insert(component_name, component);
                return Ok(true);
            }
            None => return Ok(false),
        };
    }

    // Don't Care pins
    // <component name> <list of pin indexes>
    fn parse_dont_care(&mut self, line: &str) -> Result<bool, String> {
        match DONT_CARE_FULL_RX.captures(line) {
            Some(captures) => {
                let component_name = captures[1].to_string();
                let pin_idx_list = captures[2].to_string();
                let component = self
                    .layout
                    .circuit
                    .component_name_to_component_map
                    .get_mut(&component_name);
                return match component {
                    Some(component) => {
                        let package_pos_vec = self
                            .layout
                            .circuit
                            .package_to_pos_map
                            .get(&component.package_name)
                            .unwrap();
                        for pin_idx_str in WHITESPACE_SEP_RX.split(&pin_idx_list.as_str()) {
                            let pin_idx = pin_idx_str.parse::<usize>().unwrap();
                            if pin_idx < 1 || pin_idx > package_pos_vec.len() {
                                return Err(format!(
                                    "Invalid \"Don't Care\" pin number for {}: {}. \
                                    Must be between 1 and {} (including)",
                                    component_name,
                                    pin_idx,
                                    package_pos_vec.len()
                                ));
                            }
                            component.dont_care_pin_idx_set.insert(pin_idx - 1);
                        }
                        Ok(true)
                    }
                    None => Err(format!("Unknown component: {}", component_name)),
                };
            }
            None => return Ok(false),
        }
    }

    // Connection
    // 7400.9 rpi.10
    fn parse_connection(&mut self, line: &str) -> Result<bool, String> {
        return match CONNECTION_FULL_RX.captures(line) {
            Some(captures) => {
                let start = ConnectionPoint::new(captures[1].to_string(), captures[2].parse::<usize>().unwrap() - 1);
                let end = ConnectionPoint::new(captures[3].to_string(), captures[4].parse::<usize>().unwrap() - 1);
                self.check_connection_point(&start)?;
                self.check_connection_point(&end)?;
                if start.component_name != end.component_name || start.pin_idx != end.pin_idx {
                    self.layout.circuit.connection_vec.push(Connection::new(start, end));
                }
                Ok(true)
            }
            None => Ok(false),
        };
    }

    fn check_connection_point(&self, connection_point: &ConnectionPoint) -> Result<(), String> {
        let component = self
            .layout
            .circuit
            .component_name_to_component_map
            .get(&connection_point.component_name);
        match component {
            Some(component) => {
                let package_pos_vec = self
                    .layout
                    .circuit
                    .package_to_pos_map
                    .get(&component.package_name)
                    .unwrap();
                let pin_idx_1_base = connection_point.pin_idx + 1;
                if pin_idx_1_base < 1 || pin_idx_1_base > package_pos_vec.len() {
                    return Err(format!(
                        "Invalid pin number for {}.{}. Must be between 1 and {} (including)",
                        connection_point.component_name,
                        pin_idx_1_base,
                        package_pos_vec.len()
                    ));
                }
                if component.dont_care_pin_idx_set.contains(&connection_point.pin_idx) {
                    return Err(format!(
                        "Invalid pin number for {}.{}. Pin has been set as \"Don't Care\"",
                        connection_point.component_name, pin_idx_1_base
                    ));
                }
                Ok(())
            }
            None => Err(format!("Unknown component: {}", connection_point.component_name)),
        }
    }
}
