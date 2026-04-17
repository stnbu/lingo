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
    vocab: String,
    reading: String,
    translation: String,
    is_front: bool,
    conn: Connection,
    mode: i64,
    random: bool,
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
            vocab: String::new(),
            reading: String::new(),
            translation: String::new(),
            conn,
            is_front: true,
            mode: 1,
            random: false,
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
        self.vocab = v.vocab;
        self.reading = v.reading;
        self.translation = v.translation;
        self.is_front = true;
    }

    fn write_result(&self, result: bool) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.conn.execute(
            "INSERT INTO results (vocab_id, result, datetime, mode) VALUES (?1, ?2, ?3, ?4)",
            params![&self.id, result, now, &self.mode],
        )?;
        Ok(())
    }

    fn random_vocab_id(&self) -> Result<Option<i64>> {
        let mut stmt = self.conn.prepare(
            r#"
              SELECT id
              FROM vocab
              WHERE id IN (
                  SELECT id
                  FROM vocab
                  ORDER BY RANDOM()
                  LIMIT 1
              );
              "#,
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
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
                    AND r.mode = ?1
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
                ( ?2 - COALESCE(last_seen, 0) ) DESC,

                -- randomness to break ties
                RANDOM()
            LIMIT 1
            "#,
        )?;

        let mut rows = stmt.query([&self.mode, &now])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }
}

impl eframe::App for LingoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::bottom("bottom_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        match match (ui.button("Pass").clicked(), ui.button("Fail").clicked()) {
                            (true, false) => Some(true),
                            (false, true) => Some(false),
                            _ => None,
                        } {
                            Some(result) => {
                                self.write_result(result).unwrap();
                                let id = if self.random {
                                    self.random_vocab_id().unwrap().unwrap()
                                } else {
                                    self.next_vocab_id().unwrap().unwrap()
                                };
                                self.get_vocab(id);
                            }
                            None => {}
                        };
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mode:");
                        ui.radio_value(&mut self.mode, 1, "No Reading");
                        ui.radio_value(&mut self.mode, 2, "Reading");
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.random, "Random");
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Flip").clicked() {
                            self.flip();
                        }
                    });
                })
            });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let front = match &self.mode {
                    1 => self.vocab.clone(),
                    2 => format!("{}\n{}", &self.vocab, &self.reading),
                    _ => "ERR".to_string(),
                };
                let back = match &self.mode {
                    1 => format!("{}\n{}", &self.reading, &self.translation),
                    2 => self.translation.clone(),
                    _ => "ERR".to_string(),
                };
                ui.label(egui::RichText::new(if self.is_front { front } else { back }).size(80.0));
            });
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
