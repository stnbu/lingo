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
    id: i64,
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
            id: 1,
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
        let id = s.next_vocab_id().unwrap().unwrap();
        s.get_vocab(id);
        s
    }

    fn flip(&mut self) {
        self.is_front = !self.is_front;
    }

    fn get_vocab(&mut self, id: i64) {
        let mut stmt = self
            .conn
            .prepare("SELECT id, vocab, reading, translation FROM vocab WHERE id = ?1")
            .unwrap();
        let v = stmt
            .query_row([id], |row| {
                Ok(Vocab {
                    id: row.get(0).unwrap(),
                    vocab: row.get(1).unwrap(),
                    reading: row.get(2).unwrap(),
                    translation: row.get(3).unwrap(),
                })
            })
            .unwrap();
        self.id = v.id;
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

    fn next_vocab_id(&self) -> Result<Option<i64>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut stmt = self.conn.prepare(
            r#"
            WITH stats AS (
                SELECT
                    v.id AS vocab_id,
                    COUNT(r.id) AS total,
                    SUM(CASE WHEN r.result = 0 THEN 1 ELSE 0 END) AS failures,
                    MAX(r.datetime) AS last_seen
                FROM vocab v
                LEFT JOIN results r ON v.id = r.vocab_id
                GROUP BY v.id
            )
            SELECT vocab_id
            FROM stats
            ORDER BY
                -- prioritize unseen cards
                (total = 0) DESC,

                -- higher failure ratio first
                (CAST(failures AS REAL) / NULLIF(total, 0)) DESC,

                -- older cards first
                ( ? - COALESCE(last_seen, 0) ) DESC,

                -- randomness to break ties
                RANDOM()
            LIMIT 1
            "#,
        )?;

        let mut rows = stmt.query([now])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }
}

impl eframe::App for LingoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel")
            .resizable(true)
            .min_size(32.0)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                    ui.label(egui::RichText::new(if self.is_front {
                        &self.front
                    } else {
                        &self.back
                    }).size(50.0));
                });
            });
        egui::Panel::bottom("bottom_panel")
            .resizable(false)
            .min_size(0.0)
            .show_inside(ui, |ui| {
                if ui.button("Pass").clicked() {
                    self.write_result(self.id, true).unwrap();
                    let id = self.next_vocab_id().unwrap().unwrap();
                    self.get_vocab(id);
                }
                if ui.button("Fail").clicked() {
                    self.write_result(self.id, false).unwrap();
                    let id = self.next_vocab_id().unwrap().unwrap();
                    self.get_vocab(id);
                }
                if ui.button("Flip").clicked() {
                    self.flip();
                }
            });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "lingo",
        options,
        Box::new(|cc| Ok(Box::new(LingoApp::new(cc)))),
    )
}
