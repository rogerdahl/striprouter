#![allow(unused)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod board;
pub mod circuit;
mod circuit_parser;
mod ga_core;
mod ga_interface;
mod layout;
mod nets;
mod render;
mod router;
mod router_thread;
mod settings;
mod status;
mod thread_stop;
mod ucs;
mod via;

// // Equivalent of #include <algorithm>, <chrono>, <climits>, <cstdio>, <ctime>,
// // <iostream>, <mutex>, <string>, <thread>, <vector>
// // Equivalent of using namespace std::chrono_literals; use std::time::Duration;
// use std::thread;
// use std::sync::{Arc, Mutex};
// use std::cmp;
// use std::io::{self, Write};
// use std::vec::Vec;
// use std::string::String;

static CIRCUIT_FILE_PATH: &'static str = "/home/dahl/dev/rust/striprouter/circuits/example.circuit";

use std::sync::{Arc, Mutex};
use via::{LayerStartEndVia, LayerVia, Pos, ValidVia, Via};

use crate::ga_interface::GeneticAlgorithm;
use crate::layout::Layout;
use crate::render::Render;
use crate::router_thread::RouterThread;
use crate::thread_stop::ThreadStop;
use crate::via::StartEndVia;
use eframe::egui;
use eframe::emath::Align2;
use eframe::epaint::Shape;
use egui::introspection::font_id_ui;
use egui::{Pos2, TextStyle};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([2000.0, 2000.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Stripboard Autorouter",
        options,
        Box::new(|cc| {
            // This gives us image support:
            // egui_extras::install_image_loaders(&cc.egui_ctx);

            // setup_custom_fonts(&cc.egui_ctx);

            Box::<MyApp>::default()
            // Box::<MyApp>::MyApp::new(cc)
        }),
    )
}

struct MyApp {
    // name: String,
    // age: u32,
    layout: Layout,
    zoom: f32,
    // thread_stop_router: ThreadStop,
    // router_thread: router_thread::RouterThread,
}

// impl MyApp {
//     fn new(cc: &eframe::CreationContext<'_>) -> Self {
//         setup_custom_fonts(&cc.egui_ctx);
//         Self {
//             name: "Arthur".to_owned(),
//             age: 42,
//         }
//     }
// }

impl Default for MyApp {
    fn default() -> Self {
        let mut layout = Layout::new();
        circuit_parser::CircuitFileParser::new(&mut layout).parse(CIRCUIT_FILE_PATH);

        let mut router = router::Router::new(layout.board);
        let mut nets = nets::Nets::new(layout.board);

        // let settings = settings::Settings::new();

        // router.route(
        //     layout.board,
        //     &mut layout,
        //     &mut nets,
        //     Via::new(usize::MAX, usize::MAX),
        // );

        layout.via_set_vec = nets.via_set_vec;
        layout.set_idx_vec = nets.set_idx_vec;

        Self {
            //         // name: "Arthur".to_owned(),
            //         // age: 42,
            layout,
            zoom: 25.0,
            // thread_stop_router,
            // router_thread: router_thread::RouterThread::new(ThreadStop::new()),
        }
    }
}

impl eframe::App for MyApp {
    // fn setup(&mut self, ctx: &egui::Context) {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // This scales all UI elements
        ctx.set_pixels_per_point(1.5);

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

        // Exit if the Escape key is pressed
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            std::process::exit(0);
        }

        // UI elements

        egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
            // Parse a circuit file
            ui.label("Parse a circuit file");
            if ui.button("Parse").clicked() {
                use std::time::Instant;
                let start = Instant::now();

                circuit_parser::CircuitFileParser::new(&mut self.layout).parse(CIRCUIT_FILE_PATH);

                let duration = start.elapsed();
                println!("Time elapsed in expensive_function() is: {:?}", duration);
            }

            ui.heading("Status");
            ui.label("Controls");

            // ui.horizontal(|ui| {
            //     let name_label = ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut self.name)
            //         .labelled_by(name_label.id);
            // });
            ui.add(egui::Slider::new(&mut self.zoom, 1.0..=100.0).text("Zoom"));
            // ui.label(format!("Hello '{}', age {}", self.name, self.age));

            ui.heading("Router");
            ui.label("Controls");

        });

        // Stripboard

        egui::CentralPanel::default().show(ctx, |ui| {
            use egui::{Color32, FontId, RichText};

            // ui.label("Normal text");
            // ui.label(RichText::new("Large text").font(FontId::proportional(40.0)));
            // ui.label(RichText::new("Red text").color(Color32::RED));ui.label("Stripboard");

            // let top_left = self.to_pos(ui.min_rect().left_top());
            let mut render = Render::new(self.zoom);
            render.start_render(ctx);

            // let x = RichText::new("Large text").font(FontId::proportional(40.0));
            // let sx = Shape::text();
            // // Shape::text(Pos::new(10.0, 10.0), x, (), Default::default(), Default::default(), Default::default()).fill(Color32::RED);
            // ui.painter().add(sx);

            // let pos = ui.input(|input| input.pointer.hover_pos());
            // ui.label(format!("Mouse position: {:?}", pos));

            // let pos = ui.input().pointer.screen_pos();
            // ui.label(format!("Mouse position: {:?}", pos));

            render.draw(ctx, ui, &self.layout, true, false);

            // Create a sub ui:
            // ui.horizontal(|ui| {
            //     // ui.fonts(|fonts| {
            //     //     let font = fonts.get(font_id);
            //     //     let font_size = font.size;
            //     //     let text_style = TextStyle::Body;
            //     //     let galley =
            //     //         font.layout_multiline(text_style, "Hello World!", ui.available_width());
            //     //     let pos = ui.cursor().min.translate((0.0, font_size * 1.0).into());
            //     //     let text_color = ui.visuals().text_color();
            //     //     ui.painter().galley(pos, galley);
            //     // });
            //
            //     setup_custom_fonts(ctx);
            //
            //     let font_id = TextStyle::Body.resolve(ui.style());
            //
            //     ui.fonts(|f| {
            //         let text_style = TextStyle::Body;
            //         // let galley =
            //         //     font.layout_multiline(text_style, "Hello World!", ui.available_width());
            //         let s = Shape::text(
            //             f,
            //             Pos2::new(100.0, 100.0),
            //             Align2::LEFT_BOTTOM,
            //             "XXXXXXXXXXXXXXXX",
            //             font_id,
            //             ui.visuals().text_color(),
            //         );
            //
            //         ui.painter().add(s);
            //     });
            // });

            // ui.fonts(|f| {
            //     let s = Shape::text(
            //         f,
            //         Pos2::new(100.0, 100.0),
            //         Align2::LEFT_BOTTOM,
            //         "XXXXXXXXXXXXXXXX",
            //         font_id,
            //         ui.visuals().text_color(),
            //     );
            //
            //     ui.painter().add(s);
            //
            // });

            render.end_render(ctx);
        });
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "/home/dahl/.fonts/Roboto/hinted/Roboto-Regular.ttf"
        )),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

// Equivalent of #include <GL/glew.h>, <GLFW/glfw3.h>
// You'll need to add glfw and glew to your Cargo.toml
// extern crate glfw;
// use glfw::{Action, Context, Key};

// extern crate gl;
// use gl::types::*;

// Equivalent of #include <cmdparser.hpp>, <fmt/format.h>, <nanogui/nanogui.h>
// You'll need to add cmdparser, fmt, and nanogui to your Cargo.toml
// extern crate cmdparser;
// extern crate fmt;
// extern crate nanogui;

// Equivalent of #include "circuit_parser.h", "circuit_writer.h", "ga_interface.h",
// "gl_error.h", "gui.h", "gui_status.h", "icon.h", "ogl_text.h", "render.h",
// "router.h", "status.h", "utils.h", "via.h", "write_svg.h"
// You'll need to create these modules in Rust
// mod circuit_parser;
// mod circuit_writer;
// mod ga_interface;
// mod gl_error;
// mod gui;
// mod gui_status;
// mod icon;
// mod ogl_text;
// mod render;
// mod router;
// mod status;
// mod utils;
// mod via;
// mod write_svg;

// Equivalent of std::string DIAG_FONT_PATH = "./fonts/RobotoMono-Regular.ttf"; static DIAG_FONT_PATH: &'static str = "./fonts/RobotoMono-Regular.ttf";

// Equivalent of const int DIAG_FONT_SIZE = 12; const DIAG_FONT_SIZE: usize = 12;

// Equivalent of const int DRAG_FONT_SIZE = 12; const DRAG_FONT_SIZE: usize = 12;

// Equivalent of OglText diagText(DIAG_FONT_PATH, DIAG_FONT_SIZE);
// You'll need to define the OglText struct in Rust
// const diag_text = ogl_text::OglText::new(DIAG_FONT_PATH, DIAG_FONT_SIZE);

// Equivalent of const float ZOOM_MOUSE_WHEEL_STEP = 0.3f; const ZOOM_MOUSE_WHEEL_STEP: f32 = 0.3;

// Equivalent of const float ZOOM_MIN = -1.0f; const ZOOM_MIN: f32 = -1.0;

// Equivalent of const float ZOOM_MAX = 5.0f; const ZOOM_MAX: f32 = 5.0;

// Equivalent of const float ZOOM_DEF = 2.0f; const ZOOM_DEF: f32 = 2.0;

// Equivalent of const int INITIAL_BORDER_PIXELS = 50; const INITIAL_BORDER_PIXELS: usize = 50;

// // Equivalent of float zoomLinear = ZOOM_DEF;
// let zoom_linear = ZOOM_DEF;
//
// // Equivalent of float zoom = expf(zoomLinear);
// let zoom: f32 = zoom_linear.exp();
//
// // Equivalent of bool isZoomPanAdjusted = false;
// let is_zoom_pan_adjusted: bool = false;

// Equivalent of void setZoomPan(Pos scrPos);
// You'll need to define the Pos struct in Rust fn set_zoom_pan(scr_pos: Pos) {
// Implementation goes here
// }

// Equivalent of void centerBoard(); fn center_board() {
// Implementation goes here
// }

// // Equivalent of nanogui::Slider* zoomSlider;
// // You'll need to define the Slider struct in Rust
// let zoom_slider: Option<Slider> = None;
//
// // Equivalent of int windowW = 1920 / 2;
// let window_w: usize = 1920 / 2;
//
// // Equivalent of int windowH = 1080 - 200;
// let window_h: usize = 1080 - 200;

// Equivalent of const int WINDOW_WIDTH_MIN = 500; const WINDOW_WIDTH_MIN: usize = 500;

// Equivalent of const int WINDOW_HEIGHT_MIN = 500; const WINDOW_HEIGHT_MIN: usize = 500;

// // Equivalent of bool isShowRatsNestEnabled = true;
// let is_show_rats_nest_enabled: bool = true;
//
// // Equivalent of bool isShowOnlyFailedEnabled = true;
// let is_show_only_failed_enabled: bool = true;
//
// // Equivalent of bool isShowCurrentEnabled = false;
// let is_show_current_enabled: bool = false;

// Equivalent of glm::mat4x4 projMat;
// You'll need to add glm to your Cargo.toml
// extern crate glm;
// let proj_mat: glm::Mat4 = glm::Mat4::identity();
//
// // Equivalent of nanogui::FormHelper* form;
// // You'll need to define the FormHelper struct in Rust
// let form: Option<FormHelper> = None;
//
// // Equivalent of nanogui::Window* formWin;
// // You'll need to define the Window struct in Rust
// let form_win: Option<Window> = None;
//
// // Equivalent of nanogui::Button* saveInputLayoutButton;
// // You'll need to define the Button struct in Rust
// let save_input_layout_button: Option<Button> = None;
//
// // Equivalent of GuiStatus guiStatus;
// // You'll need to define the GuiStatus struct in Rust
// let gui_status = GuiStatus::new();

// Equivalent of const int N_ROUTER_THREADS = std::thread::hardware_concurrency();
// const N_ROUTER_THREADS: usize = num_cpus::get();
//
// // Equivalent of std::vector<std::thread> routerThreadVec(N_ROUTER_THREADS);
// let router_thread_vec: Vec<Option<thread::JoinHandle<()>>> = vec![None; N_ROUTER_THREADS];
//
// // Equivalent of ThreadStop threadStopRouter;
// // You'll need to define the ThreadStop struct in Rust
// let thread_stop_router = ThreadStop::new();

// Equivalent of void stopRouterThreads(); fn stop_router_threads() {
// Implementation goes here
// }

// Equivalent of void routerThread(); fn router_thread() {
// Implementation goes here
// }

// Equivalent of void launchRouterThreads(); fn launch_router_threads() {
// Implementation goes here
// }

// Equivalent of std::thread parserThreadObj;
// let parser_thread_obj: Option<thread::JoinHandle<()>> = None;

// Equivalent of ThreadStop threadStopParser;
// let thread_stop_parser = ThreadStop::new();

// Equivalent of void stopParserThread(); fn stop_parser_thread() {
// Implementation goes here
// }

// Equivalent of void parserThread(); fn parser_thread() {
// Implementation goes here
// }

// Equivalent of void launchParserThread(); fn launch_parser_thread() {
// Implementation goes here
// }

// Equivalent of bool isComponentDragActive = false;
// let is_component_drag_active: bool = false;
//
// // Equivalent of bool isBoardDragActive = false;
// let is_board_drag_active: bool = false;
//
// // Equivalent of Pos dragStartPos;
// let drag_start_pos = Pos::new();
//
// // Equivalent of Pos panOffsetScrPos;
// let pan_offset_scr_pos = Pos::new();
//
// // Equivalent of Pos dragPin0BoardOffset;
// let drag_pin0_board_offset = Pos::new();
//
// // Equivalent of std::string dragComponentName;
// let drag_component_name = String::new();
//
// // Equivalent of OglText dragText(DIAG_FONT_PATH, DRAG_FONT_SIZE);
// let drag_text = ogl_text::OglText::new(DIAG_FONT_PATH, DRAG_FONT_SIZE);

// Equivalent of void handleMouseDragOperations(const IntPos& mouseScrPos);
// You'll need to define the IntPos struct in Rust fn handle_mouse_drag_operations(mouse_scr_pos: &IntPos) {
// Implementation goes here
// }

// Equivalent of void renderDragStatus(IntPos mouseScrPos); fn render_drag_status(mouse_scr_pos: IntPos) {
// Implementation goes here
// }

// Equivalent of Layout inputLayout;
// You'll need to define the Layout struct in Rust
// let input_layout = Layout::new();
//
// // Equivalent of Layout currentLayout;
// let current_layout = Layout::new();
//
// // Equivalent of Layout bestLayout;
// let best_layout = Layout::new();

// Equivalent of const int N_ORGANISMS_IN_POPULATION = 1000; const N_ORGANISMS_IN_POPULATION: usize = 1000;

// Equivalent of const double CROSSOVER_RATE = 0.7; const CROSSOVER_RATE: f64 = 0.7;

// Equivalent of const double MUTATION_RATE = 0.01; const MUTATION_RATE: f64 = 0.01;

// Equivalent of GeneticAlgorithm geneticAlgorithm(N_ORGANISMS_IN_POPULATION, CROSSOVER_RATE, MUTATION_RATE);
// You'll need to define the GeneticAlgorithm struct in Rust
// let genetic_algorithm = GeneticAlgorithm::new(N_ORGANISMS_IN_POPULATION, CROSSOVER_RATE, MUTATION_RATE);

// Equivalent of Render render;
// You'll need to define the Render struct in Rust
// let render = Render::new();

// Equivalent of void resetInputLayout(); fn reset_input_layout() {
// Implementation goes here
// }

// Equivalent of nanogui::Button* saveBestLayoutButton;
// let save_best_layout_button: Option<Button> = None;

// Equivalent of Status status;
// You'll need to define the Status struct in Rust
// let status = Status::new();

// Equivalent of void printStats(); fn print_stats() {
// Implementation goes here
// }

// Equivalent of TrackAverage averageRenderTime(60);
// You'll need to define the TrackAverage struct in Rust
// let average_render_time = TrackAverage::new(60);

// Equivalent of TrackAverage averageFailedRoutes(N_ORGANISMS_IN_POPULATION);
// let average_failed_routes = TrackAverage::new(N_ORGANISMS_IN_POPULATION);

// Equivalent of ThreadStop threadStopApp;
// let thread_stop_app = ThreadStop::new();

// Equivalent of void runHeadless(); fn run_headless() {
// Implementation goes here
// }

// Equivalent of void runGui(); fn run_gui() {
// Implementation goes here
// }

// Equivalent of void exitApp(); fn exit_app() {
// Implementation goes here
// }

// Equivalent of volatile bool isParserPaused = true;
// let is_parser_paused: Arc<Mutex<bool>> = Arc::new(Mutex::new(true));

// Equivalent of void parseCommandLineArgs(int argc, char** argv); fn parse_command_line_args(args: Vec<String>) {
// Implementation goes here
// }

// // Equivalent of bool noGui;
// let no_gui: bool = false;
//
// // Equivalent of bool useRandomSearch;
// let use_random_search: bool = false;
//
// // Equivalent of bool exitOnFirstComplete;
// let exit_on_first_complete: bool = false;
//
// // Equivalent of long exitAfterNumChecks;
// let exit_after_num_checks: usize = 0;
//
// // Equivalent of long checkpointAtNumChecks;
// let checkpoint_at_num_checks: usize = 0;
//
// // Equivalent of std::string circuitFilePath;
// let circuit_file_path: String = String::new(); mod via;
