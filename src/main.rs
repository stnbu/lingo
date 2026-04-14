#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use rusqlite::{Connection, Result};

pub struct Vocab {
    pub id: i64,
    pub vocab: String,
    pub reading: String,
    pub translation: String,
}

pub fn get_vocab_by_id(path: &str, id: i64) -> Result<Vocab> {
    let conn = Connection::open(path)?;

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

/*
fn main() -> anyhow::Result<()> {
    let db_path = "vocab.db";
    // /Users/mburr/Downloads/vocab.db
    let v = db::get_vocab_by_id(db_path, 1)?;
    println!("{} {} {} {}", v.id, v.vocab, v.reading, v.translation);
    Ok(())
}
*/

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
