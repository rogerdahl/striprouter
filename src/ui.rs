use crate::render::Render;
use crate::MyApp;
use eframe::epaint::{Color32, FontId};
use egui::{Context, FontData, FontDefinitions, FontFamily, RichText, TextStyle, Ui};
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
        // let mut style: egui::Style = (*ctx.style()).clone();

        // style.visuals.extreme_bg_color = egui::Color32::from_rgb(45, 51, 59);
        // style.visuals.faint_bg_color = egui::Color32::from_rgb(45, 51, 59);
        // style.visuals.code_bg_color = egui::Color32::from_rgb(45, 51, 59);
        // style.visuals.hyperlink_color = egui::Color32::from_rgb(255, 0, 0);
        // style.visuals.override_text_color = Some(egui::Color32::from_rgb(173, 186, 199));
        // // style.visuals.window_corner_radius = 10.0;
        // style.visuals.button_frame = true;
        // style.visuals.collapsing_header_frame = true;
        // style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(35, 39, 46);
        // style.visuals.widgets.noninteractive.fg_stroke =
        //     egui::Stroke::new(0., egui::Color32::from_rgb(173, 186, 199));
        // style.visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        // style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 51, 59);
        // style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(45, 51, 59);
        // style.visuals.widgets.open.bg_fill = egui::Color32::from_rgb(45, 51, 59);
        // ctx.set_style(style);

        // let mut fonts = egui::FontDefinitions::default();
        // // let font_data = std::include_bytes!("../fonts/RobotoMono-Regular.ttf").to_vec();
        // // let font = egui::Font::from_bytes(font_data);
        // // let font_id = fonts..fonts_for_family.get_mut(&egui::TextStyle::Monospace).unwrap();
        // // *font_id = Some(font);
        // ctx.set_fonts(fonts);

        // ctx.begin_frame();

        // The collection of fonts used by epaint.
        //
        // Required in order to paint text. Create one and reuse. Cheap to clone.
        //
        // Each Fonts comes with a font atlas textures that needs to be used when painting.
        //
        // If you are using egui, use egui::Context::set_fonts and egui::Context::fonts.
        //
        // You need to call Self::begin_frame and Self::font_image_delta once every frame.

        // Drawing fancy labels
        // ui.label(RichText::new("Large text").font(FontId::proportional(40.0)));
        // ui.label(RichText::new("Red text").color(Color32::RED));

        egui::SidePanel::left("control_panel")
            // .frame()
            .show(ctx, |ui| {
                ui.add_space(8.0);

                ui.vertical_centered(|ui| {
                    ui.heading("Status");
                });

                ui.scope(|ui| {
                    ui.style_mut().visuals.indent_has_left_vline = false;
                    ui.style_mut().spacing.indent_ends_with_horizontal_line = false;
                    // ui.style_mut().spacing.indent = Spacing::new(0.0, 0.0);

                    Controls::level_0_label(ui, "Render");
                    ui.indent(1, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("ms/frame");
                            Controls::highlighted_label(ui, &format!("{}", self.ms_per_frame));
                        });
                    });

                    Controls::level_0_label(ui, "Total");
                    ui.indent(1, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Checked");
                            Controls::highlighted_label(ui, &format!("{}", self.checked_total));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Checked/s");
                            Controls::highlighted_label(ui, &format!("{}", self.checked_per_second));
                        });
                    });

                    Controls::level_0_label(ui, "Current Layout");
                    ui.indent(1, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Completed");
                            Controls::highlighted_label(ui, &format!("{}", self.ms_per_frame));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Failed Avg");
                            Controls::highlighted_label(ui, &format!("{}", self.failed_routes_avg));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Cost");
                            Controls::highlighted_label(ui, &format!("{}", self.layout_current_cost));
                        });
                    });

                    Controls::level_0_label(ui, "Best Layout");
                    ui.indent(1, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Completed");
                            Controls::highlighted_label(ui, &format!("{}", self.best_layout_completed_routes));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Failed");
                            Controls::highlighted_label(ui, &format!("{}", self.best_layout_failed_routes));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Cost");
                            Controls::highlighted_label(ui, &format!("{}", self.best_layout_cost));
                        });
                    });

                    ui.add_space(8.0);

                    ui.vertical_centered(|ui| {
                        ui.heading("Router");
                    });

                    Controls::level_0_label(ui, "Costs");
                    ui.indent(1, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Wire");
                            ui.add(egui::DragValue::new(&mut self.wire_cost).clamp_range(0..=100));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Strip");
                            ui.add(egui::DragValue::new(&mut self.strip_cost).clamp_range(0..=100));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Via");
                            ui.add(egui::DragValue::new(&mut self.via_cost).clamp_range(0..=100));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Cut");
                            ui.add(egui::DragValue::new(&mut self.cut_cost).clamp_range(0..=100));
                        });
                    });

                    Controls::level_0_label(ui, "Display");
                    ui.indent(1, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Rat's Nest");
                            ui.checkbox(&mut self.show_rats_nest, "");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Only Failed");
                            ui.checkbox(&mut self.show_only_failed, "");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Current");
                            ui.checkbox(&mut self.show_current_layout, "");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Zoom");
                            // ui.set_max_width(100.0);
                            // ui.add_sized(ui.available_size(), widget
                            ui.add_sized(
                                ui.available_size(),
                                egui::Slider::new(&mut self.zoom, 1.0..=100.0).show_value(false),
                            );
                        });
                    });

                    Controls::level_0_label(ui, "Misc");
                    ui.indent(1, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Pause");
                            ui.checkbox(&mut self.pause_router, "");
                        });
                    });

                    Controls::level_0_label(ui, "Input Layout");
                    ui.indent(1, |ui| {
                        // ui.add_sized(ui.available_size(), Button::new("Save to .circuit file"));
                        if ui.button("Save to .circuit file").clicked() {
                            println!("Button was clicked!");
                        }
                    });

                    Controls::level_0_label(ui, "Best Layout");
                    ui.indent(1, |ui| {
                        // ui.fill(|ui| {
                        if ui.button("Save to .svg files").clicked() {
                            println!("Button was clicked!");
                        }
                        // });
                    });

                    // Modify the style within this scope
                    // ui.style_mut().visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(255, 0, 0);
                    // ui.style_mut().visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 255, 0);
                    // ui.style_mut().visuals.widgets.bg_fill = egui::Color32::from_rgb(0, 255, 0);
                    // [TextStyle::Body].glyphs.size(label_text)
                    // let (id, rect) = ui.allocate_space(label_size);
                });

                // egui::Grid::new("_options").num_columns(4).show(ui, |ui| {
                //     // ui.label("Override text style:");
                //     // // crate::ComboBox::from_id_source("Override text style").selected_text(match override_text_style {
                //     // //     None => "None".to_owned(),
                //     // //     Some(override_text_style) => override_text_style.to_string(),
                //     // // }).show_ui(ui, |ui| {
                //     // //     ui.selectable_value(override_text_style, None, "None");
                //     // //     let all_text_styles = ui.style().text_styles();
                //     // //     for style in all_text_styles {
                //     // //         let text = crate::RichText::new(style.to_string()).text_style(style.clone());
                //     // //         ui.selectable_value(override_text_style, Some(style), text);
                //     // //     }
                //     // // });
                //     //
                //     // ui.horizontal(|ui| {
                //     //     // ui.radio_value(override_font_id, None, "None");
                //     //     // if ui.radio(override_font_id.is_some(), "override").clicked() {
                //     //     //     *override_font_id = Some(FontId::default());
                //     //     // }
                //     //     // if let Some(override_font_id) = override_font_id {
                //     //     //     crate::introspection::font_id_ui(ui, override_font_id);
                //     // });
                //     //
                //     // ui.end_row();
                // });
            });
    }

    fn level_0_label(ui: &mut Ui, s: &str) {
        ui.add_space(8.0);
        let text_color = ui.visuals().strong_text_color();
        let font_id = TextStyle::Button.resolve(ui.style());
        let galley = ui.fonts(|fonts| fonts.layout(s.to_string(), font_id, text_color, ui.available_width()));
        let (id, rect) = ui.allocate_space(galley.size());
        ui.painter().galley(rect.min, galley);
    }

    fn highlighted_label(ui: &mut Ui, s: &str) {
        let text_color = ui.visuals().strong_text_color();
        let font_id = TextStyle::Button.resolve(ui.style());
        let galley = ui.fonts(|fonts| fonts.layout(s.to_string(), font_id, text_color, ui.available_width()));
        let (id, rect) = ui.allocate_space(galley.size());
        // let color = Color32::from_white_alpha(128);
        let color = ui.visuals().weak_text_color();
        ui.painter().rect_filled(rect, 0.0, color);
        ui.painter().galley(rect.min, galley);
    }
}

fn setup_custom_fonts(ctx: &Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        FontData::from_static(include_bytes!("/home/dahl/.fonts/Roboto/hinted/Roboto-Regular.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}
