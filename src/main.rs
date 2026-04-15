#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily};
use rusqlite::{Connection, Result};
use std::sync::Arc;

pub struct Vocab {
    pub id: i64,
    pub vocab: String,
    pub reading: String,
    pub translation: String,
}

struct Content {
    front: String,
    back: String,
    is_front: bool,
    conn: Connection,
}

fn load_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "hiragino".to_owned(),
        Arc::new(FontData::from_owned(
            std::fs::read("/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc").unwrap(),
        )),
    );

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "hiragino".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "hiragino".to_owned());

    ctx.set_fonts(fonts);
}

impl Default for Content {
    fn default() -> Self {
        let conn = Connection::open("lingo.db").unwrap();
        Self {
            front: String::new(),
            back: String::new(),
            conn,
            is_front: true,
        }
    }
}

impl Content {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_fonts(&cc.egui_ctx);
        Self::default()
    }
}

impl eframe::App for Content {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if ui.button("Pass").clicked() {
                let v = get_vocab(&self.conn, 1).unwrap();
                self.front = v.vocab;
            }
            if ui.button("Fail").clicked() {
                self.front = "fail".to_owned();
            }
            if ui.button("Flip").clicked() {
                println!("flip");
            }
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.label(&self.front);
                });
        });
    }
}

pub fn get_vocab(conn: &Connection, id: i64) -> Result<Vocab> {
    let mut stmt =
        conn.prepare("SELECT id, vocab, reading, translation FROM vocab WHERE id = ?1")?;
    let v = stmt.query_row([id], |row| {
        Ok(Vocab {
            id: row.get(0)?,
            vocab: row.get(1)?,
            reading: row.get(2)?,
            translation: row.get(3)?,
        })
    })?;
    Ok(v)
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "lingo",
        options,
        Box::new(|cc| Ok(Box::new(Content::new(cc)))),
    )
}
