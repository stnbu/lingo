#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use rusqlite::{Connection, Result};

pub struct Vocab {
    pub id: i64,
    pub vocab: String,
    pub reading: String,
    pub translation: String,
}

#[derive(Default)]
struct Content {
    text: String,
}

impl eframe::App for Content {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let path = "lingo.db";
        let conn = Connection::open(path).unwrap();
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if ui.button("Pass").clicked() {
                let v = get_vocab(&conn, 1).unwrap();
                self.text = format!("{} {} {} {}", v.id, v.vocab, v.reading, v.translation);
            }
            if ui.button("Fail").clicked() {
                self.text = "fail".to_owned();
            }
            if ui.button("Flip").clicked() {
                println!("flip");
            }

            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.label(&self.text);
                });

        });
    }
}

pub fn get_vocab(conn: &Connection, id: i64) -> Result<Vocab> {

    let mut stmt = conn.prepare(
        "SELECT id, vocab, reading, translation FROM vocab WHERE id = ?1"
    )?;

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
        Box::new(|_cc| Ok(Box::<Content>::default())),
    )
}
