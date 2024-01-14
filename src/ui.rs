use crate::render::Render;
use crate::MyApp;
use eframe::epaint::{Color32, FontId};
use egui::{Button, Context, FontData, FontDefinitions, FontFamily, RichText, TextStyle, Ui};
use std::sync::atomic::Ordering;
use std::time::Instant;

pub(crate) struct Controls {
    pub ms_per_frame: f32,
    pub checked_total: f32,
    pub checked_per_second: f32,
    pub wire_cost: i32,
    pub strip_cost: i32,
    pub via_cost: i32,
    pub cut_cost: i32,
    pub zoom: f32,
    pub failed_routes_avg: usize,
    pub layout_current_cost: usize,
    pub best_layout_completed_routes: usize,
    pub best_layout_failed_routes: usize,
    pub best_layout_cost: usize,
    pub show_rats_nest: bool,
    pub show_only_failed: bool,
    pub show_current_layout: bool,
    pub pause_router: bool,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            ms_per_frame: 1.0,
            checked_total: 0.0,
            checked_per_second: 0.0,
            wire_cost: 1,
            strip_cost: 1,
            via_cost: 1,
            cut_cost: 1,
            zoom: 15.0,
            failed_routes_avg: 0,
            layout_current_cost: 0,
            best_layout_completed_routes: 0,
            best_layout_failed_routes: 0,
            best_layout_cost: 0,
            show_rats_nest: false,
            show_only_failed: false,
            show_current_layout: false,
            pause_router: false,
        }
    }
}

impl Controls {
    pub(crate) fn ui(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("control_panel").show(ctx, |ui| {
            ui.scope(|ui| {
                ui.style_mut().visuals.indent_has_left_vline = false;
                ui.style_mut().spacing.indent_ends_with_horizontal_line = false;

                ui.add_space(8.0);

                ui.vertical_centered(|ui| {
                    ui.heading("File");
                });

                Controls::level_0_label(ui, "Circuit file");
                ui.indent(1, |ui| {
                    egui::Grid::new("file-circuit-grid").show(ui, |ui| {
                        // ui.add_sized(ui.available_size(), Button::new("Save to .circuit file"));
                        if ui.button("Open").clicked() {
                            println!("Button was clicked!");
                        }
                        if ui.button("Save").clicked() {
                            println!("Button was clicked!");
                        }
                        ui.end_row();
                    });
                });

                Controls::level_0_label(ui, "Best Layout");
                ui.indent(1, |ui| {
                    egui::Grid::new("file-svg-grid").show(ui, |ui| {
                        if ui.button("Save to .svg files").clicked() {
                            println!("Button was clicked!");
                        }
                    });
                });

                ui.add_space(16.0);

                ui.vertical_centered(|ui| {
                    ui.heading("Status");
                });

                Controls::level_0_label(ui, "Total");
                ui.indent(1, |ui| {
                    egui::Grid::new("total-grid").show(ui, |ui| {
                        ui.label("Checked");
                        Controls::highlighted_label(ui, &format!("{}", self.checked_total));
                        ui.end_row();

                        ui.label("Checked/s");
                        Controls::highlighted_label(ui, &format!("{}", self.checked_per_second));
                        ui.end_row();
                    });
                });

                Controls::level_0_label(ui, "Current Layout");
                ui.indent(1, |ui| {
                    egui::Grid::new("current-cost-grid").show(ui, |ui| {
                        ui.label("Completed");
                        Controls::highlighted_label(ui, &format!("{}", self.ms_per_frame));
                        ui.end_row();

                        ui.label("Failed Avg");
                        Controls::highlighted_label(ui, &format!("{}", self.failed_routes_avg));
                        ui.end_row();

                        ui.label("Cost");
                        Controls::highlighted_label(ui, &format!("{}", self.layout_current_cost));
                        ui.end_row();
                    });
                });

                Controls::level_0_label(ui, "Best Layout");
                ui.indent(1, |ui| {
                    egui::Grid::new("best-cost-grid").show(ui, |ui| {
                        ui.label("Completed");
                        Controls::highlighted_label(ui, &format!("{}", self.best_layout_completed_routes));
                        ui.end_row();

                        ui.label("Failed");
                        Controls::highlighted_label(ui, &format!("{}", self.best_layout_failed_routes));
                        ui.end_row();

                        ui.label("Cost");
                        Controls::highlighted_label(ui, &format!("{}", self.best_layout_cost));
                        ui.end_row();
                    });
                });

                Controls::level_0_label(ui, "Render");
                ui.indent(1, |ui| {
                    egui::Grid::new("render-grid").show(ui, |ui| {
                        ui.label("ms/frame");
                        Controls::highlighted_label(ui, &format!("{}", self.ms_per_frame));
                        ui.end_row();
                    });
                });

                ui.add_space(16.0);

                ui.vertical_centered(|ui| {
                    ui.heading("Router");
                });

                Controls::level_0_label(ui, "Costs");
                ui.indent(1, |ui| {
                    egui::Grid::new("costs-grid").show(ui, |ui| {
                        ui.label("Wire");
                        ui.add(egui::DragValue::new(&mut self.wire_cost).clamp_range(0..=100));
                        ui.end_row();

                        ui.label("Strip");
                        ui.add(egui::DragValue::new(&mut self.strip_cost).clamp_range(0..=100));
                        ui.end_row();

                        ui.label("Via");
                        ui.add(egui::DragValue::new(&mut self.via_cost).clamp_range(0..=100));
                        ui.end_row();

                        ui.label("Cut");
                        ui.add(egui::DragValue::new(&mut self.cut_cost).clamp_range(0..=100));
                        ui.end_row();
                    });
                });

                Controls::level_0_label(ui, "Display");
                ui.indent(1, |ui| {
                    egui::Grid::new("display-grid").show(ui, |ui| {
                        ui.label("Rat's Nest");
                        ui.checkbox(&mut self.show_rats_nest, "");
                        ui.end_row();

                        ui.label("Only Failed");
                        ui.checkbox(&mut self.show_only_failed, "");
                        ui.end_row();

                        ui.label("Current");
                        ui.checkbox(&mut self.show_current_layout, "");
                        ui.end_row();

                        ui.label("Zoom");
                        ui.add_sized(
                            ui.available_size(),
                            egui::Slider::new(&mut self.zoom, 1.0..=100.0).show_value(false),
                        );
                        ui.end_row();
                    });
                });

                Controls::level_0_label(ui, "Misc");
                ui.indent(1, |ui| {
                    egui::Grid::new("misc-grid").show(ui, |ui| {
                        ui.label("Pause");
                        ui.checkbox(&mut self.pause_router, "");
                        ui.end_row();
                    });
                });
            });
        });
    }

    fn level_0_label(ui: &mut Ui, s: &str) {
        ui.add_space(6.0);
        let text_color = ui.visuals().strong_text_color();
        let font_id = TextStyle::Button.resolve(ui.style());
        let galley = ui.fonts(|fonts| fonts.layout(s.to_string(), font_id, text_color, ui.available_width()));
        let (id, rect) = ui.allocate_space(galley.size());
        ui.painter().galley(rect.min, galley);
        // Right align
        // let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
        // let font = ui.fonts()[TextStyle::Body];
        // let galley = ui.painter().layout_multiline(font, self.text.clone(), ui.available_width());
        // let pos = response.rect.right_top() - vec2(galley.size().x, 0.0);
        // painter.galley(pos, galley);
    }

    fn highlighted_label(ui: &mut Ui, s: &str) {
        let text_color = ui.visuals().strong_text_color();
        let font_id = TextStyle::Button.resolve(ui.style());
        let galley = ui.fonts(|fonts| fonts.layout(format!("  { }  ", s), font_id, text_color, ui.available_width()));
        let (id, rect) = ui.allocate_space(galley.size());
        // let color = Color32::from_white_alpha(128);
        let color = ui.visuals().weak_text_color();
        ui.painter().rect_filled(rect, 2.0, color);
        ui.painter().galley(rect.min, galley);
    }
}
