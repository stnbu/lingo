#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_ui_native("My egui App", options, move |ctx, _frame| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            ui.heading("My egui Application");
            if ui.button("Pass").clicked() {
                println!("pass");
            }
            if ui.button("Fail").clicked() {
                println!("fail");
            }
            if ui.button("Flip").clicked() {
                println!("flip");
            }
        });
    })
}
