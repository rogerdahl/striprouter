#![allow(unused)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use eframe::egui;
use rand::Rng;

use crate::layout::Layout;
use crate::render::Render;
// use crate::thread_stop::ThreadStop;
use crate::router_control::RouterControl;

mod board;
pub mod circuit;
mod circuit_parser;
mod controls;
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
mod util;
mod via;

static CIRCUIT_FILE_PATH: &'static str = "../../circuits/example.circuit";

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 800.0]),
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
    counter: Arc<AtomicUsize>,
    start: Instant,

    controls: controls::Controls,

    zoom: f32,
}

// impl MyApp {
//     fn new(cc: &eframe::CreationContext<'_>) -> Self {
//         setup_custom_fonts(&cc.egui_ctx);
//         Self {
//         }
//     }
// }

impl Default for MyApp {
    fn default() -> Self {
        let input_layout = Arc::new(Mutex::new(Layout::new()));
        let current_layout = Arc::new(Mutex::new(Layout::new()));
        let best_layout = Arc::new(Mutex::new(Layout::new()));

        let mut bin_path = env::current_exe().unwrap();
        bin_path.pop();
        bin_path.push(CIRCUIT_FILE_PATH);
        circuit_parser::CircuitFileParser::new(&mut input_layout.lock().unwrap()).parse(bin_path.as_os_str());

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

        let zoom = 15.0;

        Self {
            input_layout,
            current_layout,
            best_layout,
            router_control,
            // router_stop_signal: thread_stop,
            counter: counter.clone(),
            start: Instant::now(),

            zoom,

            controls: controls::Controls::new(
                0.0, 0, 0.0, 0, 0, 0, 0,
                zoom, 0, 0, 0, 0, 0, 0, false, false, false, false),
        }
    }
}

impl eframe::App for MyApp {
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
            // let top_left = self.to_pos(ui.min_rect().left_top());
            let mut render = Render::new(self.controls.zoom);
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
