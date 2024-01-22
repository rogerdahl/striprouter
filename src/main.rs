#![allow(unused)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use eframe::egui;
use rand::Rng;

use crate::controls::Controls;
use crate::layout::Layout;
use crate::render::Render;
// use crate::thread_stop::ThreadStop;
use crate::router_control::RouterControl;
use crate::status::Status;

mod board;
pub mod circuit;
mod circuit_parser;
mod controls;
mod ga_core;
mod ga_interface;
mod layout;
mod nets;
mod render;
mod render_ascii;
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

    status: Status,
    // controls: controls::Controls<'a>,

    limit_routes: Arc<AtomicUsize>,
}

// impl MyApp {
//     fn new(cc: &eframe::CreationContext<'_>) -> Self {
//         setup_custom_fonts(&cc.egui_ctx);
//         Self {
//         }
//     }
// }

impl<'a> Default for MyApp {
    fn default() -> Self {
        let input_layout = Arc::new(Mutex::new(Layout::new()));
        let current_layout = Arc::new(Mutex::new(Layout::new()));
        let best_layout = Arc::new(Mutex::new(Layout::new()));

        let mut bin_path = env::current_exe().unwrap();
        bin_path.pop();
        bin_path.push(CIRCUIT_FILE_PATH);
        circuit_parser::CircuitFileParser::new(&mut input_layout.lock().unwrap()).parse(bin_path.as_os_str());

        let limit_routes = Arc::new(AtomicUsize::new(0));
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

        // TODO: Second clone needed here?
        let mut router_control = RouterControl::new(
            Arc::clone(&input_layout.clone()),
            Arc::clone(&current_layout.clone()),
            Arc::clone(&best_layout),
            counter.clone(),
            Arc::clone(&limit_routes),
            // Arc::clone(&current_layout),
        );

        router_control.start();

        let mut zoom = 15.0;

        Self {
            input_layout,
            current_layout,
            best_layout,
            router_control,
            // router_stop_signal: thread_stop,
            counter: counter.clone(),
            start: Instant::now(),
            status: Status::new(),
            // controls: Controls::new(
            //     0.0, 0, 0.0, 0, 0, 0, 0,
            //     &mut 0.0, 0, 0, 0, 0, 0, 0, false, false, false, false),

            limit_routes,
        }

        // app.controls = Controls::new(
        //         0.0, 0, 0.0, 0, 0, 0, 0,
        //         &mut 0.0, 0, 0, 0, 0, 0, 0, false, false, false, false),
        // )
    }
}

impl<'a> eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Exit if the Escape key is pressed
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            std::process::exit(0);
        }

        // This scales all UI elements
        ctx.set_pixels_per_point(1.3);

        let mut controls = Controls::new();

        // Controls
        // TODO: Don't want to keep the lock for the whole render.
        let mut best_layout = self.best_layout.lock().unwrap().clone();

        // let mut x = &best_layout;
        // println!("x.layout_info_vec.len() = {}", x.layout_info_vec.len());
        // println!("x.route_status_vec.len() = {}", x.route_status_vec.len());
        // println!("x.set_idx_vec.len() = {}", x.set_idx_vec.len());
        // println!("x.strip_cut_vec.len() = {}", x.strip_cut_vec.len());
        // println!("x.via_set_vec.len() = {}", x.via_set_vec.len());
        // println!("x.route_vec.len() = {}", x.route_vec.len());
        // println!("x.n_completed_routes = {}", x.n_completed_routes);
        // println!("x.n_failed_routes = {}", x.n_failed_routes);
        // println!("x.cost = {}", x.cost);

        self.status.best_layout_cost = best_layout.cost;
        controls.render(ctx, &mut self.status, &mut self.limit_routes);

        let mut input_layout = self.input_layout.lock().unwrap().clone();

        // println!("via_cost: {}", best_layout.settings.via_cost);
        // Stripboard
        egui::CentralPanel::default().show(ctx, |ui| {
            // let top_left = self.to_pos(ui.min_rect().left_top());
            let mut render = Render::new(self.status.zoom);
            render.start_render(ctx);

            // let pos = ui.input().pointer.screen_pos();
            // ui.label(format!("Mouse position: {:?}", pos));

            // {
            //     let _timer = Timer::new();
            // render.draw(ctx, ui, &input_layout, true, false);
            render.draw(ctx, ui, &best_layout, false, false);


            // let render_ascii = render_ascii::RenderAscii::new(60, 40);
            // render_ascii.draw(&best_layout);

            // if self.start.elapsed().as_millis() > 1000 {
            //     println!(
            //         "best_layout: cost={:?} completed= {:?}",
            //         best_layout.cost, best_layout.n_completed_routes
            //     );
            //     self.start = Instant::now();
            //     println!("counter: {:?}", self.counter.load(Ordering::SeqCst));
            //     self.counter.store(0, Ordering::SeqCst);
            // }

            render.end_render(ctx);

            // Request a repaint at the end of the update
            ctx.request_repaint();
        });
    }
}
