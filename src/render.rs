use std::f32::consts::PI;
use std::sync::Mutex;

// Assuming equivalent Rust crates for these C++ libraries are being used
// use glew::Context as GlewContext;
// use glm::{Mat4, Vec4};
// use gui::Gui;
// use render::Render;
// use shader::Shader;

const PI_F: f32 = PI;
const CIRCUIT_FONT_SIZE: f32 = 1.0;
const CIRCUIT_FONT_PATH: &str = "./fonts/Roboto-Regular.ttf";
const NOTATION_FONT_SIZE: i32 = 10;
const SET_DIM: f32 = 0.3;
const NUM_VIA_TRIANGLES: i32 = 16;
const CUT_WIDTH: f32 = 0.83;
const VIA_RADIUS: f32 = 0.2;
const WIRE_WIDTH: f32 = 0.125;
const RATS_NEST_WIRE_WIDTH: f32 = 0.1;
const CONNECTION_WIDTH: f32 = 0.1;

// pub struct Render {
//     // component_text: OglText,
//     // notation_text: OglText,
//     fill_program_id: u32,
//     vertex_buf_id: u32,
// }
//
// impl Render {
//     pub fn new() -> Self {
//         Self {
//             // component_text: OglText::new(CIRCUIT_FONT_PATH, CIRCUIT_FONT_SIZE),
//             // notation_text: OglText::new(CIRCUIT_FONT_PATH, NOTATION_FONT_SIZE),
//             fill_program_id: 0,
//             vertex_buf_id: 0,
//         }
//     }
//
//     // ... rest of the methods translated to Rust ...
//
//     // pub fn set_color(&self, rgba: Vec4) {
//     //     unsafe {
//     //         // gl::UseProgram(self.fill_program_id);
//     //         // let color_loc = gl::GetUniformLocation(self.fill_program_id, c_str!("color"));
//     //         // gl::Uniform4fv(color_loc, 1, rgba.as_ptr());
//     //     }
//     // }
//
//     // ... rest of the methods translated to Rust ...
// }


use egui::*;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]

pub struct Painting {
    /// in 0-1 normalized coordinates
    lines: Vec<Vec<Pos2>>,
    stroke: Stroke,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
        }
    }
}

impl Painting {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            egui::stroke_ui(ui, &mut self.stroke, "Stroke");
            ui.separator();
            if ui.button("Clear Painting").clicked() {
                self.lines.clear();
            }
        })
        .response
    }

    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        if self.lines.is_empty() {
            self.lines.push(vec![]);
        }

        let current_line = self.lines.last_mut().unwrap();

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos = from_screen * pointer_pos;
            if current_line.last() != Some(&canvas_pos) {
                current_line.push(canvas_pos);
                response.mark_changed();
            }
        } else if !current_line.is_empty() {
            self.lines.push(vec![]);
            response.mark_changed();
        }

        let shapes = self
            .lines
            .iter()
            .filter(|line| line.len() >= 2)
            .map(|line| {
                let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
                egui::Shape::line(points, self.stroke)
            });

        painter.extend(shapes);

        response
    }
}

impl super::Demo for Painting {
    fn name(&self) -> &'static str {
        "🖊 Painting"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        use super::View as _;
        Window::new(self.name())
            .open(open)
            .default_size(vec2(512.0, 512.0))
            .vscroll(false)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl super::View for Painting {
    fn ui(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add(crate::egui_github_link_file!());
        });
        self.ui_control(ui);
        ui.label("Paint with your mouse/touch!");
        Frame::canvas(ui.style()).show(ui, |ui| {
            self.ui_content(ui);
        });
    }
}
