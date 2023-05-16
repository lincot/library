#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use dotenvy::dotenv;
use eframe::egui;
use gui::Library;
use std::fs::create_dir_all;

fn main() -> Result<(), eframe::Error> {
    dotenv().ok();
    env_logger::init();
    create_dir_all("books").unwrap();
    create_dir_all("covers").unwrap();
    eframe::run_native(
        "Library",
        eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(320.0, 240.0)),
            ..Default::default()
        },
        Box::new(|_cc| Box::<Library>::default()),
    )
}
