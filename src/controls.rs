use crate::render::Render;
use crate::{status, MyApp};
use eframe::epaint::{Color32, FontId};
use egui::WidgetText::RichText;
use egui::{Align, Button, Context, FontData, FontDefinitions, FontFamily, Rect, Sense, TextStyle, Ui};
use num_format::{Locale, ToFormattedString};
use std::fmt::format;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub(crate) struct Controls {}

impl<'a> Controls {
    pub(crate) fn new() -> Self {
        Self {}
    }

    // #[rustfmt::skip::attributes(max_width)]
    #[rustfmt::skip]
    pub(crate) fn render(
        &mut self, ctx: &Context,
        status: &mut status::Status,
        limit_routes: &mut Arc<AtomicUsize>
    ) {
        egui::SidePanel::left("control_panel").show(ctx, |ui| {
            ui.scope(|ui| {
                ui.style_mut().visuals.indent_has_left_vline = false;
                ui.style_mut().spacing.indent_ends_with_horizontal_line = false;

                ui.add_space(8.0);

                egui::Grid::new("control-grid").show(ui, |ui| {
                    Controls::header(ui, "File", true);
                    Controls::section(ui, "Circuit file");

                    ui.horizontal(|ui| {
                        ui.label("    ");
                        if ui.button("Open").clicked() {
                            println!("Button was clicked!");
                        }
                    });
                    ui.end_row();

                    ui.horizontal(|ui| {
                        ui.label("    ");
                        if ui.button("Save").clicked() {
                            println!("Button was clicked!");
                        }
                    });
                    ui.end_row();

                    Controls::section(ui, "Best Layout");
                    ui.horizontal(|ui| {
                        ui.label("    ");
                        if ui.button("Save to .svg").clicked() {
                            println!("Button was clicked!");
                        }
                    });
                    ui.end_row();

                    Controls::header(ui, "Layouts", false);

                    Controls::section(ui, "Total");

                    Controls::name_int(ui, "Checked", status.checked_total);
                    Controls::name_float(ui, "Checked/s", status.checked_per_second);

                    Controls::section(ui, "Current");

                    Controls::name_int(ui, "Completed", status.current_layout_completed_routes);
                    Controls::name_int(ui, "Failed Avg", status.current_layout_failed_routes);
                    Controls::name_int(ui, "Cost", status.current_layout_cost);

                    Controls::section(ui, "Best");

                    Controls::name_int(ui, "Completed", status.best_layout_completed_routes);
                    Controls::name_int(ui, "Failed", status.best_layout_failed_routes);
                    Controls::name_int(ui, "Cost", status.best_layout_cost);

                    Controls::header(ui, "Router", false);

                    Controls::section(ui, "Costs");

                    Controls::name_widget(ui, "Wire", egui::DragValue::new(&mut status.wire_cost).clamp_range(1..=100));
                    Controls::name_widget(ui, "Strip", egui::DragValue::new(&mut status.strip_cost).clamp_range(1..=100));
                    Controls::name_widget(ui, "Via", egui::DragValue::new(&mut status.via_cost).clamp_range(1..=100));//.speed(0.01));

                    ui.end_row();

                    let mut limit_routes_ = limit_routes.load(Ordering::SeqCst);

                    ui.label(format!("LIMIT={} ", limit_routes_));

                    ui.horizontal(|ui| {
                        ui.label("    ");
                        if ui.button("<").clicked() {
                            limit_routes_ -= 1;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("    ");
                        if ui.button(">").clicked() {
                            limit_routes_ += 1;
                        }
                    });

                    limit_routes.store(limit_routes_, Ordering::SeqCst);

                    ui.end_row();


                    Controls::name_widget(ui, "Cut", egui::DragValue::new(&mut status.cut_cost).clamp_range(1..=100));

                    Controls::section(ui, "Display");

                    Controls::name_widget(ui, "Rat's Nest", egui::Checkbox::new(&mut status.show_rats_nest, ""));
                    Controls::name_widget(ui, "Current", egui::Checkbox::new(&mut status.show_only_failed, ""));
                    Controls::name_widget(ui, "Only Failed", egui::Checkbox::new(&mut status.show_current_layout, ""));

                    Controls::section(ui, "Misc");

                    Controls::name_widget(ui, "Zoom", egui::DragValue::new(&mut status.zoom).clamp_range(1..=100));
                    Controls::name_widget(ui, "Pause", egui::Checkbox::new(&mut status.pause_router, ""));
                    Controls::name_float(ui, "ms/frame", status.ms_per_frame);
                });
            });
        });
    }

    fn header(ui: &mut Ui, s: &str, first: bool) {
        if !first {
            ui.end_row();
        }
        ui.vertical(|ui| {
            ui.heading(s);
            ui.add_space(ui.style().spacing.item_spacing.y / 2.0);
        });
        ui.end_row();
    }

    fn section(ui: &mut Ui, s: &str) {
        let text_color = ui.visuals().strong_text_color();
        let font_id = TextStyle::Button.resolve(ui.style());
        let galley = ui.fonts(|fonts| fonts.layout(s.to_string(), font_id, text_color, ui.available_width()));
        ui.vertical(|ui| {
            ui.add_space(ui.style().spacing.item_spacing.y * 2.0);
            let (id, rect) = ui.allocate_space(galley.size());
            ui.painter().galley(rect.min, galley, Color32::BLACK);
        });
        ui.end_row();
    }

    fn name_int(ui: &mut Ui, name: &str, v: usize) {
        ui.label(format!("    {}", name));
        Controls::highlighted_label(ui, &format!("{}", v.to_formatted_string(&Locale::en)));
        ui.end_row();
    }

    fn name_float(ui: &mut Ui, name: &str, value: f32) {
        ui.label(format!("    {}", name));
        Controls::highlighted_label(ui, &format!("{:.2}", value));
        ui.end_row();
    }

    fn name_widget(ui: &mut Ui, name: &str, widget: impl egui::Widget) {
        ui.label(format!("    {}", name));
        ui.add(widget);
        ui.end_row();
    }

    // Draw a non-interactive value using a disabled button widget
    fn highlighted_label(ui: &mut Ui, s: &str) {
        let prefix = "  ";
        let suffix = "  ";
        let text_style = ui.style().drag_value_text_style.clone();
        let button = Button::new(egui::RichText::new(format!("{}{}{}", prefix, s, suffix)).text_style(text_style))
            .wrap(false)
            .sense(Sense::focusable_noninteractive())
            .min_size(ui.spacing().interact_size);
        ui.add(button);
    }

    // Draw a value using RichText widget.
    // fn highlighted_label(ui: &mut Ui, s: &str) {
    //     ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
    //         let bg_color = ui.visuals().weak_text_color();
    //         ui.horizontal(|ui| {
    //             // This adds space to the right, since we're in a right_to_left block.
    //             // ui.add_space(ui.style().spacing.item_spacing.y);
    //             // Couldn't find this value, so this is a hack to match spacing on
    //             // right of DragValue widgets.
    //             ui.add_space(8.0);
    //             let rich_text = egui::widgets::Label::new(
    //                 // The spaces on the sides of the value add padding to the left and
    //                 // right sides of the background color.
    //                 egui::RichText::from(format!("  { }  ", s))
    //                     .background_color(bg_color)
    //                     .strong()
    //             );
    //             ui.add(rich_text);
    //         });
    //     });
    // }

    // Attempt at prettier version, but the rect size is incorrect on wider values.
    // fn highlighted_label(ui: &mut Ui, s: &str) {
    //     ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
    //         let text_color = ui.visuals().strong_text_color();
    //         let font_id = TextStyle::Button.resolve(ui.style());
    //         let galley =
    //             ui.fonts(|fonts| fonts.layout(format!("  { }  ", s), font_id, text_color, ui.available_width()));
    //         let (id, rect) = ui.allocate_space(galley.size());
    //         let color = ui.visuals().weak_text_color();
    //         ui.painter().rect_filled(Rect::from_min_max(rect.left_top(), rect.right_bottom()), 2.0, color);
    //         ui.painter().galley(rect.min, galley);
    //         });
    //     });
    // }
}
