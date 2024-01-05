use std::f32::consts::PI;

use egui::*;

use crate::gui;
use crate::layout::Layout;
use crate::via::{Pos, StartEndVia, ValidVia, Via};

// const CIRCUIT_FONT_SIZE: f32 = 1.0;
// // const CIRCUIT_FONT_PATH: &str = "/home/dahl/.fonts/Roboto/hinted/Roboto-Regular.ttf";
// const NOTATION_FONT_SIZE: usize = 10;
const CUT_WIDTH: f32 = 0.83;
const VIA_RADIUS: f32 = 0.2;
const WIRE_WIDTH: f32 = 0.125;
const RATS_NEST_WIRE_WIDTH: f32 = 0.2;
const CONNECTION_WIDTH: f32 = 0.2;
const LABEL_SIZE_POINT: f32 = 0.75;
const DIAG_SIZE_POINT: f32 = 0.25;

pub struct Render {
    label_font_id: FontId,
    diag_font_id: FontId,
    zoom: f32,
    mouse_board_pos: Pos,

    strip_regular_color: Color32,
    strip_dimmed_color: Color32,
    strip_via_color: Color32,

    wire_regular_color: Color32,
    wire_dimmed_color: Color32,

    strip_cut_color: Color32,

    component_color: Color32,
    component_pin_color: Color32,
    component_dont_care_pin_color: Color32,
    component_name_color: Color32,

    rats_nest_unrouted_color: Color32, // not yet routed
    rats_nest_success_color: Color32,  // successfully routed
    rats_nest_failed_color: Color32,   // failed routing

    border_color: Color32,

    diag_wire_layer_color: Color32,
    diag_strip_layer_color: Color32,
    diag_wire_cost_color: Color32,
    diag_strip_cost_color: Color32,
    diag_start_pos_color: Color32,
    diag_end_pos_color: Color32,

    notation_color: Color32,
}

impl Render {
    pub fn new(zoom: f32) -> Self {
        Self {
            label_font_id: FontId::new(zoom * LABEL_SIZE_POINT, FontFamily::Monospace),
            diag_font_id: FontId::new(zoom * DIAG_SIZE_POINT, FontFamily::Monospace),
            zoom,
            mouse_board_pos: Pos::new(0.0, 0.0),

            strip_regular_color: Self::color(0.85, 0.565, 0.345, 1.0),
            strip_dimmed_color: Self::color(0.85 * 0.3, 0.565 * 0.3, 0.345 * 0.3, 1.0),
            strip_via_color: Self::color(0.0, 0.0, 0.0, 1.0),

            wire_regular_color: Self::color(0.7, 0.7, 0.7, 1.0),
            wire_dimmed_color: Self::color(0.3, 0.3, 0.3, 1.0),

            strip_cut_color: Self::color(0.0, 0.8, 0.8, 1.0),

            component_color: Self::color(0.0, 0.0, 0.0, 0.4),
            component_pin_color: Self::color(0.784, 0.0, 0.0, 1.0),
            component_dont_care_pin_color: Self::color(0.0, 0.784, 0.0, 1.0),
            component_name_color: Self::color(0.0, 0.0, 0.0, 1.0),

            rats_nest_unrouted_color: Self::color(0.0, 0.392, 0.784, 0.5),
            rats_nest_success_color: Self::color(0.0, 0.584, 0.192, 0.5),
            rats_nest_failed_color: Self::color(0.784, 0.3, 0.0, 0.5),

            border_color: Self::color(0.0, 0.0, 0.0, 1.0),

            diag_wire_layer_color: Self::color(1.0, 0.0, 0.0, 1.0),
            diag_strip_layer_color: Self::color(0.0, 1.0, 0.0, 1.0),
            diag_wire_cost_color: Self::color(1.0, 0.0, 0.0, 1.0),
            diag_strip_cost_color: Self::color(0.0, 1.0, 0.0, 1.0),
            diag_start_pos_color: Self::color(1.0, 1.0, 1.0, 1.0),
            diag_end_pos_color: Self::color(1.0, 1.0, 1.0, 1.0),

            notation_color: Self::color(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn start_render(&mut self, ctx: &Context) {
        // self.fonts
        //     .begin_frame(self.height_in_points, self.max_texture_size);
    }

    pub fn end_render(&mut self, ctx: &Context) {
        // self.fonts.font_image_delta();
    }

    pub fn draw(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        // board: Board,
        layout: &Layout,
        // board_screen_offset: &Pos,
        // mouse_board_pos: &Pos,
        // window_w: usize,
        // window_h: usize,
        show_rats_nest: bool,
        show_only_failed: bool,
    ) {
        let top_left = self.to_pos(ui.min_rect().left_top());
        let mouse_via = self.get_mouse_via(ui, layout, top_left);
        // println!("mouse_via: {:?}", mouse_via);

        let mouse_net = self.get_net(ui, layout, &mouse_via);
        let is_mouse_on_net = !mouse_net.is_empty();

        self.draw_strip_sections(ui, layout, &mouse_net, is_mouse_on_net);
        self.draw_wire_sections(ui, layout, &mouse_net, is_mouse_on_net);
        self.draw_strip_cuts(ui, layout);
        self.draw_components(ui, layout);
        if show_rats_nest {
            self.draw_rats_nest(ui, layout, show_only_failed);
        }
        // self.draw_border(ui, layout);
        if layout.has_error {
            self.draw_diag(ui, layout);
        }
    }

    pub fn draw_strip_sections(
        &self,
        ui: &mut Ui,
        layout: &Layout,
        mouse_net: &Vec<Via>,
        is_mouse_on_net: bool,
    ) {
        // Routes
        // Draw strips and wires separately so that wires are always on top.
        // Strips

        for route_section_vec in &layout.route_vec {
            for section in route_section_vec {
                let start = &section.start.via;
                let end = &section.end.via;
                assert_eq!(section.start.is_wire_layer, section.end.is_wire_layer);
                if !section.start.is_wire_layer {
                    let start_end_via = &StartEndVia::new(*start, *end);
                    let mut y1 = start_end_via.start.y;
                    let mut y2 = start_end_via.end.y;
                    if y1 > y2 {
                        std::mem::swap(&mut y1, &mut y2);
                    }
                    let start1 = Pos::new(
                        start_end_via.start.x as f32 - CUT_WIDTH / 2.0,
                        y1 as f32 - 0.40,
                    );
                    let end1 = Pos::new(
                        start_end_via.start.x as f32 + CUT_WIDTH / 2.0,
                        y2 as f32 + 0.40,
                    );
                    let mut color;
                    if is_mouse_on_net && !mouse_net.contains(&section.start.via) {
                        color = self.strip_dimmed_color;
                    } else {
                        color = self.strip_regular_color;
                    }
                    self.draw_filled_rectangle(ui, start1, end1, &color);
                    // Vias
                    for y in y1..=y2 {
                        self.draw_filled_circle(
                            ui,
                            Pos::new(start_end_via.start.x as f32, y as f32),
                            VIA_RADIUS,
                            &self.strip_via_color,
                        );
                    }
                }
            }
        }
    }

    pub fn draw_wire_sections(
        &self,
        ui: &mut Ui,
        layout: &Layout,
        mouse_net: &Vec<Via>,
        is_mouse_on_net: bool,
    ) {
        for route_section_vec in &layout.route_vec {
            for section in route_section_vec {
                let start = &section.start.via;
                let end = &section.end.via;
                if start.x != end.x && start.y == end.y {
                    let mut x1 = section.start.via.x;
                    let mut x2 = section.end.via.x;
                    if x1 > x2 {
                        std::mem::swap(&mut x1, &mut x2);
                    }
                    let mut color;
                    if is_mouse_on_net && !mouse_net.contains(&section.start.via) {
                        color = self.wire_dimmed_color;
                    } else {
                        color = self.wire_regular_color;
                    }
                    self.draw_thick_line(
                        ui,
                        section.start.via.cast::<f32>(),
                        section.end.via.cast::<f32>(),
                        CONNECTION_WIDTH,
                        &color,
                    );
                    self.draw_filled_circle(ui, section.start.via.cast::<f32>(), VIA_RADIUS, &color);
                    self.draw_filled_circle(ui, section.end.via.cast::<f32>(), VIA_RADIUS, &color);
                }
            }
        }
    }

    pub fn draw_strip_cuts(&self, ui: &mut Ui, layout: &Layout) {
        for v in &layout.strip_cut_vec {
            let half_strip_w = CUT_WIDTH / 2.0;
            let half_cut_h = 0.08 / 2.0;
            let start = Pos::new((v.x as f32 - half_strip_w), (v.y as f32 - half_cut_h));
            let end = Pos::new((v.x as f32 + half_strip_w), (v.y as f32 + half_cut_h));
            self.draw_filled_rectangle(
                ui,
                start - Pos::new(0.0, 0.5),
                end - Pos::new(0.0, 0.5),
                &self.strip_cut_color,
            );
        }
    }

    pub fn draw_components(&self, ui: &mut Ui, layout: &Layout) {
        for (component_name, component) in &layout.circuit.component_name_to_component_map {
            // Footprint
            let footprint = layout
                .circuit
                .calc_component_footprint(component_name.to_owned());
            let start = footprint.start.cast::<f32>() - Pos::new(0.5, 0.5);
            let end = footprint.end.cast::<f32>() + Pos::new(0.5, 0.5);
            self.draw_filled_rectangle(ui, start, end, &self.component_color);
            // Pins
            let mut is_pin0 = true;
            let mut pin_idx = 0;
            for pin_via in layout.circuit.calc_component_pins(component_name) {
                let is_dont_care_pin = component.dont_care_pin_idx_set.contains(&pin_idx);
                let color = if is_dont_care_pin {
                    self.component_dont_care_pin_color
                } else {
                    self.component_pin_color
                };
                if is_pin0 {
                    is_pin0 = false;
                    let start = pin_via.cast::<f32>() - Pos::new(VIA_RADIUS, VIA_RADIUS);
                    let end = pin_via.cast::<f32>() + Pos::new(VIA_RADIUS, VIA_RADIUS);
                    self.draw_filled_rectangle(ui, start, end, &color);
                } else {
                    self.draw_filled_circle(ui, pin_via.cast::<f32>(), VIA_RADIUS, &color);
                }
                pin_idx += 1;
            }
            // Name label
            let txt_center_b = Pos::new(
                start.x + (end.x - start.x) / 2.0,
                start.y + (end.y - start.y) / 2.0,
            );
            let txt_center_s = gui::board_to_scr_pos(&txt_center_b, self.zoom, &Pos::new(0.0, 0.0));
            self.draw_text(
                ui,
                txt_center_b,
                component_name,
                &self.component_name_color,
                true,
            );
        }
    }

    pub fn draw_rats_nest(&self, ui: &mut Ui, layout: &Layout, show_only_failed: bool) {
        let routed_con_vec = &layout.route_status_vec;
        let all_con_vec = layout.circuit.gen_connection_via_vec();
        let mut i = 0;
        for c in all_con_vec {
            let start = &c.start.cast::<f32>();
            let end = &c.end.cast::<f32>();
            if i < routed_con_vec.len() {
                // Within the routed set
                if routed_con_vec[i] {
                    if !show_only_failed {
                        self.draw_thick_line(
                            ui,
                            *start,
                            *end,
                            RATS_NEST_WIRE_WIDTH,
                            &self.rats_nest_success_color,
                        );
                    }
                } else {
                    self.draw_thick_line(
                        ui,
                        *start,
                        *end,
                        RATS_NEST_WIRE_WIDTH,
                        &self.rats_nest_failed_color,
                    );
                }
            } else {
                // Outside the routed set
                if !show_only_failed {
                    self.draw_thick_line(
                        ui,
                        *start,
                        *end,
                        RATS_NEST_WIRE_WIDTH,
                        &self.rats_nest_unrouted_color,
                    );
                }
            }
            i += 1;
        }
    }

    pub fn draw_border(&self, ui: &mut Ui, layout: &Layout) {
        let start = Pos::new(0.0, 0.0); // - 0.5;
        let end = Pos::new(layout.board.w as f32 - 1.0, layout.board.h as f32 - 1.0); // + 0.5;
        let radius = 0.2;
        self.draw_thick_line(
            ui,
            start,
            Pos::new(end.x, start.y),
            radius,
            &self.border_color,
        );
        self.draw_thick_line(
            ui,
            start,
            Pos::new(start.x, end.y),
            radius,
            &self.border_color,
        );
        self.draw_thick_line(ui, Pos::new(end.x, start.y), end, radius, &self.border_color);
        self.draw_thick_line(ui, Pos::new(start.x, end.y), end, radius, &self.border_color);
    }

    pub fn draw_diag(&self, ui: &mut Ui, layout: &Layout) {
        // Draw diag route if specified
        for v in &layout.diag_route_step_vec {
            let color = if v.is_wire_layer {
                self.diag_wire_layer_color
            } else {
                self.diag_strip_layer_color
            };
            self.draw_filled_circle(ui, v.via.cast::<f32>(), 1.0, &color);
        }
        // Draw dots where costs have been set.
        for y in 0..layout.board.h {
            for x in 0..layout.board.w {
                let idx = x + layout.board.w * y;
                let v = &layout.diag_cost_vec[idx];
                if v.wire_cost != usize::MAX {
                    self.draw_filled_circle(
                        ui,
                        Pos::new(x as f32 - 0.2, y as f32),
                        0.75,
                        &self.diag_wire_cost_color,
                    );
                }
                if v.strip_cost != usize::MAX {
                    self.draw_filled_circle(
                        ui,
                        Pos::new(x as f32 + 0.2, y as f32),
                        0.75,
                        &self.diag_strip_cost_color,
                    );
                }
            }
        }
        // Draw start and end positions if set.
        if layout.diag_start_via.is_valid {
            self.draw_filled_circle(
                ui,
                layout.diag_start_via.via.cast::<f32>(),
                1.5,
                &self.diag_start_pos_color,
            );
            self.print_notation(
                ui,
                layout.diag_start_via.via.cast::<f32>(),
                0,
                &"start".to_string(),
            );
        }
        if layout.diag_end_via.is_valid {
            self.draw_filled_circle(
                ui,
                layout.diag_end_via.via.cast::<f32>(),
                1.5,
                &self.diag_end_pos_color,
            );
            self.print_notation(
                ui,
                layout.diag_end_via.via.cast::<f32>(),
                0,
                &"end".to_string(),
            );
        }
        // Draw wire jump labels
        for y in 0..layout.board.h {
            for x in 0..layout.board.w {
                let wire_to_via =
                    &layout.diag_trace_vec[layout.board.idx(Via::new(x, y))].wire_to_via;
                if wire_to_via.is_valid {
                    self.print_notation(
                        ui,
                        Pos::new(x as f32, y as f32),
                        0,
                        &format!("->{}", wire_to_via.via.to_string()),
                    );
                }
            }
        }
        // Print error notice and any info
        let mut n_line = 0;
        let side_board_pos =
            gui::screen_to_board_pos(&Pos::new(0.0, 300.0), self.zoom, &Pos::new(0.0, 0.0));
        self.print_notation(ui, side_board_pos, n_line, &"Diag".to_string());
        n_line += 1;
        self.print_notation(ui, side_board_pos, n_line, &"wire = red".to_string());
        n_line += 1;
        self.print_notation(ui, side_board_pos, n_line, &"strip = green".to_string());
        n_line += 1;
        for str in &layout.error_string_vec {
            self.print_notation(ui, side_board_pos, n_line, str);
            n_line += 1;
        }
        // Mouse pointer coordinate info
        let v = self.get_mouse_via(ui, layout, side_board_pos); // RIGHT POS??
        if v.is_valid {
            n_line = 2;
            let idx = layout.board.idx(v.via);
            self.print_notation(ui, self.mouse_board_pos, n_line, &format!("{}", v.via));
            n_line += 1;
            let via_cost = &layout.diag_cost_vec[idx];
            self.print_notation(
                ui,
                self.mouse_board_pos,
                n_line,
                &format!("wireCost: {}", via_cost.wire_cost),
            );
            n_line += 1;
            self.print_notation(
                ui,
                self.mouse_board_pos,
                n_line,
                &format!("stripCost: {}", via_cost.strip_cost),
            );
            n_line += 1;
            let via_trace = &layout.diag_trace_vec[idx];
            self.print_notation(
                ui,
                self.mouse_board_pos,
                n_line,
                &format!("wireBlocked: {}", via_trace.is_wire_side_blocked),
            );
            n_line += 1;
            // Nets
            self.print_notation(ui, self.mouse_board_pos, n_line, &"".to_string());
            n_line += 1;
            let set_idx_size = layout.set_idx_vec.len();
            if set_idx_size > 0 {
                let set_idx = layout.set_idx_vec[idx];
                self.print_notation(
                    ui,
                    self.mouse_board_pos,
                    n_line,
                    &format!("setIdx: {}", set_idx),
                );
                n_line += 1;
                self.print_notation(
                    ui,
                    self.mouse_board_pos,
                    n_line,
                    &format!("setSize: {}", layout.via_set_vec[set_idx].len()),
                );
                n_line += 1;
            }
        }
    }

    // Get the via that the mouse is currently hovering over. Any via on the board is
    // valid. If the mouse is not in the board area, return a ValidVia with is_valid =
    // false.
    pub fn get_mouse_via(&self, ui: &mut Ui, layout: &Layout, offset_pos: Pos) -> ValidVia {
        ui.input(|input| {
            let pointer_state = &input.pointer;
            if pointer_state.has_pointer() {
                let hover_pos = self.to_pos(pointer_state.hover_pos().unwrap());
                let board_pos = gui::screen_to_board_pos(&hover_pos, self.zoom, &offset_pos);
                if board_pos.x >= 0.0 && board_pos.y >= 0.0 {
                    let via = Via::new(board_pos.x.round() as usize, board_pos.y.round() as usize);
                    let is_valid = via.x < layout.board.w && via.y < layout.board.h;
                    ValidVia { via, is_valid }
                } else {
                    ValidVia {
                        via: Via::new(0, 0),
                        is_valid: false,
                    }
                }
            } else {
                ValidVia {
                    via: Via::new(0, 0),
                    is_valid: false,
                }
            }
        })
    }

    // Determine if the mouse is hovering over a stripboard section that is used in
    // the circuit.
    pub fn get_net(&self, ui: &mut Ui, layout: &Layout, mouse_via: &ValidVia) -> Vec<Via> {
        if mouse_via.is_valid {
            for route_section_vec in &layout.route_vec {
                for section in route_section_vec {
                    let start = &section.start.via;
                    let end = &section.end.via;
                    if start.x != end.x && start.y == end.y {
                        let mouse_net = self.get_mouse_net(ui, layout, &mouse_via);
                        if !mouse_net.is_empty() && mouse_net.contains(&section.start.via) {
                            return mouse_net;
                        }
                    }
                }
            }
        }
        Vec::new()
    }

    pub fn get_mouse_net(&self, ui: &mut Ui, layout: &Layout, mouse_via: &ValidVia) -> Vec<Via> {
        let v = mouse_via;
        let empty_via_set = Vec::new();
        if !v.is_valid {
            return empty_via_set;
        }
        let idx = layout.board.idx(v.via);
        assert!(idx < layout.set_idx_vec.len());
        let set_idx = layout.set_idx_vec[idx];
        if set_idx == usize::MAX {
            empty_via_set
        } else {
            layout.via_set_vec[set_idx].clone().into_iter().collect()
        }
    }

    pub fn print_notation(&self, ui: &mut Ui, board_pos: Pos, n_line: usize, msg: &String) {
        let scr_pos = gui::board_to_scr_pos(&board_pos, self.zoom, &Pos::new(0.0, 0.0));
        self.draw_text(ui, board_pos, msg, &self.notation_color, false);
    }

    pub fn draw_filled_rectangle(&self, ui: &mut Ui, start: Pos, end: Pos, color: &Color32) {
        let top_left = self.to_pos(ui.min_rect().left_top());
        let start = gui::board_to_scr_pos(&start, self.zoom, &top_left);
        let end = gui::board_to_scr_pos(&end, self.zoom, &top_left);
        let rect = Shape::rect_filled(
            Rect::from_min_max(self.to_pos2(start), self.to_pos2(end)),
            0.0,
            *color,
        );
        ui.painter().add(rect);
    }

    pub fn draw_filled_circle(&self, ui: &mut Ui, center: Pos, radius: f32, color: &Color32) {
        let top_left = self.to_pos(ui.min_rect().left_top());
        let center = gui::board_to_scr_pos(&center, self.zoom, &top_left);
        let circle = Shape::circle_filled(self.to_pos2(center), radius * self.zoom, *color);
        ui.painter().add(circle);
    }

    pub fn draw_thick_line(&self, ui: &mut Ui, start: Pos, end: Pos, radius: f32, color: &Color32) {
        let top_left = self.to_pos(ui.min_rect().left_top());
        let start = gui::board_to_scr_pos(&start, self.zoom, &top_left);
        let end = gui::board_to_scr_pos(&end, self.zoom, &top_left);
        // println!("draw_thick_line: {:?} - {:?}", start, end);
        let line = Shape::line_segment(
            [self.to_pos2(start), self.to_pos2(end)],
            (radius * self.zoom, *color),
        );
        ui.painter().add(line);
    }

    pub fn draw_text(&self, ui: &mut Ui, pos: Pos, text: &str, color: &Color32, center: bool) {
        let top_left = self.to_pos(ui.min_rect().left_top());
        let pos = gui::board_to_scr_pos(&pos, self.zoom, &top_left);
        ui.painter().text(
            self.to_pos2(pos),
            if center {
                Align2::CENTER_CENTER
            } else {
                Align2::LEFT_CENTER
            },
            text,
            self.label_font_id.clone(),
            Color32::WHITE,
        );
    }

    // Convert from (x,y) position as used by the rest of the app, to position used by
    // egui. The position used in the rest of the app is an nalgebra vector type that
    // allows for easy math operations. The position used by egui is a simpler type that
    // is easier to work with when drawing. Both position types hold f32 values.
    pub fn to_pos2(&self, pos: Pos) -> Pos2 {
        Pos2::new(pos.x, pos.y)
    }

    // Convert from position as used by egui, to position used by the rest of the app.
    // Both position types hold f32 values.
    pub fn to_pos(&self, pos2: Pos2) -> Pos {
        Pos::new(pos2.x, pos2.y)
    }

    pub fn color(r: f32, g: f32, b: f32, a: f32) -> Color32 {
        Color32::from_rgba_unmultiplied(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        )
    }
    //     if self.is_point_outside_screen(center) {
    //         return;
    //     }
    //     self.set_color(color);
    //     let num_via_triangles = NUM_VIA_TRIANGLES;
    //     let center_s = self.board_to_scr_pos(center, self.zoom, self.board_screen_offset);
}

// impl Render {
//     pub fn ui_control(&mut self, ui: &mut Ui) -> Response {
//         ui.horizontal(|ui| {
//             stroke_ui(ui, &mut self.stroke, "Stroke");
//             ui.separator();
//             if ui.button("Clear Render").clicked() {
//                 self.lines.clear();
//             }
//         }).response
//     }
//
//     pub fn ui_content(&mut self, ui: &mut Ui) -> Response {
//         let (mut response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());
//
//         let to_screen = emath::RectTransform::from_to(
//             Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
//             response.rect,
//         );
//         let from_screen = to_screen.inverse();
//
//         if self.lines.is_empty() {
//             self.lines.push(vec![]);
//         }
//
//         let current_line = self.lines.last_mut().unwrap();
//
//         if let Some(pointer_pos) = response.interact_pointer_pos() {
//             let canvas_pos = from_screen * pointer_pos;
//             if current_line.last() != Some(&canvas_pos) {
//                 current_line.push(canvas_pos);
//                 response.mark_changed();
//             }
//         } else if !current_line.is_empty() {
//             self.lines.push(vec![]);
//             response.mark_changed();
//         }
//
//         let shapes = self.lines.iter().filter(|line| line.len() >= 2).map(|line| {
//             let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
//             Shape::line(points, self.stroke)
//         });
//
//         painter.extend(shapes);
//
//         response
//     }
// }
