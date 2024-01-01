use crate::circuit::Circuit;
use crate::via::Pos;
use std::collections::HashMap;

pub fn board_to_scr_pos(board_pos: &Pos, zoom: f32, board_screen_offset: &Pos) -> Pos {
    board_pos * zoom + board_screen_offset
}

pub fn screen_to_board_pos(scr_pos: &Pos, zoom: f32, board_screen_offset: &Pos) -> Pos {
    (scr_pos - board_screen_offset) / zoom
}

pub fn get_mouse_scr_pos(int_mouse_pos: &Pos) -> Pos {
    int_mouse_pos.cast::<f32>()
}

pub fn get_mouse_board_pos(int_mouse_pos: &Pos, zoom: f32, board_screen_offset: &Pos) -> Pos {
    screen_to_board_pos(&get_mouse_scr_pos(int_mouse_pos), zoom, board_screen_offset)
}

pub fn get_component_at_board_pos(circuit: &mut Circuit, board_pos: &Pos) -> Option<String> {
    // for (component_name, _) in &circuit.component_name_to_component_map {
    //     let footprint = circuit.calc_component_footprint(component_name);
    //     let mut start = footprint.start.cast::<f32>();
    //     let mut end = footprint.end.cast::<f32>();
    //     start -= 0.5;
    //     end += 0.5;
    //     let p = board_pos;
    //     if p.x >= start.x && p.x <= end.x && p.y >= start.y && p.y <= end.y {
    //         return Some(component_name.clone());
    //     }
    // }
    None
}

pub fn set_component_position(circuit: &mut Circuit, mouse_board_via: &Pos, component_name: &str) {
    // circuit
    //     .component_name_to_component_map
    //     .get_mut(component_name)
    //     .unwrap()
    //     .pin0_abs_pos = *mouse_board_via;
}
