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
mod router_control;
mod router_thread;
mod settings;
mod status;
mod thread_stop;
mod ucs;
mod ui;
mod util;
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

use crate::util::Timer;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use via::{LayerStartEndVia, LayerVia, Pos, ValidVia, Via};

use crate::ga_interface::GeneticAlgorithm;
use crate::layout::Layout;
use crate::render::Render;
use crate::router_thread::RouterThread;
// use crate::thread_stop::ThreadStop;
use crate::router_control::RouterControl;
use crate::via::StartEndVia;
use eframe::egui;
use eframe::emath::Align2;
use eframe::epaint::Shape;
use egui::introspection::font_id_ui;
use egui::style::Spacing;
use egui::{Pos2, TextStyle, Ui};
use rand::Rng;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 1000.0]),
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
    input_layout: Arc<Mutex<Layout>>,
    current_layout: Arc<Mutex<Layout>>,
    best_layout: Arc<Mutex<Layout>>,
    router_control: RouterControl,
    zoom: f32,

    counter: Arc<AtomicUsize>,
    start: Instant,

    controls: ui::Controls,
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
        let input_layout = Arc::new(Mutex::new(Layout::new()));
        let current_layout = Arc::new(Mutex::new(Layout::new()));
        let best_layout = Arc::new(Mutex::new(Layout::new()));

        circuit_parser::CircuitFileParser::new(&mut input_layout.lock().unwrap()).parse(CIRCUIT_FILE_PATH);

        // layout.via_set_vec = nets.via_set_vec;
        // layout.set_idx_vec = nets.set_idx_vec;

        // let thread_stop = Arc::new(Mutex::new(ThreadStop::new()));

        // let mut router_thread = RouterThread::new(
        //     thread_stop,
        //     Arc::new(Mutex::new(layout.clone())),
        //     Arc::new(Mutex::new(layout.clone())),
        // );

        // router_thread.start(Arc::new(Mutex::new(layout)));

        let counter = Arc::new(AtomicUsize::new(0));

        let mut router_control = RouterControl::new(
            Arc::clone(&input_layout),
            Arc::clone(&current_layout),
            Arc::clone(&best_layout),
            counter.clone(),
            // Arc::clone(&current_layout),
        );
        router_control.start();

        Self {
            input_layout,
            current_layout,
            best_layout,
            router_control,
            zoom: 25.0,
            // router_stop_signal: thread_stop,
            counter: counter.clone(),
            start: Instant::now(),
            controls: ui::Controls::default(),
        }
    }
}

impl eframe::App for MyApp {
    // fn setup(&mut self, ctx: &egui::Context) {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Exit if the Escape key is pressed
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            std::process::exit(0);
        }

        // This scales all UI elements
        ctx.set_pixels_per_point(1.3);

        // Controls
        self.controls.ui(ctx);

        // Stripboard
        egui::CentralPanel::default().show(ctx, |ui| {
            use egui::{Color32, FontId, RichText};


            // let top_left = self.to_pos(ui.min_rect().left_top());
            let mut render = Render::new(self.zoom);
            render.start_render(ctx);

            // let pos = ui.input().pointer.screen_pos();
            // ui.label(format!("Mouse position: {:?}", pos));

            // {
            //     let _timer = Timer::new();
            // render.draw(ctx, ui, &self.input_layout.lock().unwrap().clone(), true, false);
            let best_layout = self.best_layout.lock().unwrap().clone();
            render.draw(ctx, ui, &best_layout, true, false);

            // }
            if self.start.elapsed().as_millis() > 1000 {
                println!(
                    "best_layout: cost={:?} completed= {:?}",
                    best_layout.cost, best_layout.n_completed_routes
                );
                self.start = Instant::now();
                println!("counter: {:?}", self.counter.load(Ordering::SeqCst));
                self.counter.store(0, Ordering::SeqCst);
            }

            render.end_render(ctx);

            // Request a repaint at the end of the update
            ctx.request_repaint();
        });
    }
}
