#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily};
use rusqlite::{params, Connection, Result};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Vocab {
    pub id: i64,
    pub vocab: String,
    pub reading: String,
    pub translation: String,
}

struct LingoApp {
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

impl Default for LingoApp {
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

impl LingoApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_fonts(&cc.egui_ctx);
        let mut s = Self::default();
        s.get_vocab();
        s
    }

    fn flip(&mut self) {
        self.is_front = !self.is_front;
    }

    fn get_vocab(&mut self) {
        let mut stmt = self
            .conn
            .prepare("SELECT id, vocab, reading, translation FROM vocab WHERE id = ?1")
            .unwrap();
        let v = stmt
            .query_row([1], |row| {
                Ok(Vocab {
                    id: row.get(0).unwrap(),
                    vocab: row.get(1).unwrap(),
                    reading: row.get(2).unwrap(),
                    translation: row.get(3).unwrap(),
                })
            })
            .unwrap();
        self.front = v.vocab;
        self.back = format!("{}\n{}", v.reading, v.translation);
        self.is_front = true;
    }

    fn write_result(&self, vocab_id: i64, result: bool) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.conn.execute(
            "INSERT INTO results (vocab_id, result, datetime) VALUES (?1, ?2, ?3)",
            params![vocab_id, result, now],
        )?;
        Ok(())
    }
}

impl eframe::App for LingoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if ui.button("Pass").clicked() {
                self.write_result(1, true).unwrap();
                self.get_vocab();
            }
            if ui.button("Fail").clicked() {
                self.write_result(1, false).unwrap();
                self.get_vocab();
            }
            if ui.button("Flip").clicked() {
                self.flip();
            }
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.label(if self.is_front {
                        &self.front
                    } else {
                        &self.back
                    });
                });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "lingo",
        options,
        Box::new(|cc| Ok(Box::new(LingoApp::new(cc)))),
    )
}
