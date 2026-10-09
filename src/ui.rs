use crate::blockout::{Blockout, export_obj, load_blockout, save_blockout};
use eframe::egui;
use std::path::Path;

pub struct MapStrikeApp {
    pub blockout: Option<Blockout>,
    pub name: String,
    pub document_path: String,
    pub export_path: String,
    pub block_min: [i32; 3],
    pub block_max: [i32; 3],
    pub move_id: u32,
    pub delta: [i32; 3],
    pub status: Option<String>,
    pub error: Option<String>,
}

impl Default for MapStrikeApp {
    fn default() -> Self {
        Self {
            blockout: None,
            name: "New blockout".into(),
            document_path: String::new(),
            export_path: String::new(),
            block_min: [0, 0, 0],
            block_max: [128, 128, 64],
            move_id: 1,
            delta: [0, 0, 0],
            status: None,
            error: None,
        }
    }
}

impl MapStrikeApp {
    pub fn new_blockout(&mut self) -> Result<(), String> {
        match Blockout::new(&self.name) {
            Ok(blockout) => {
                self.blockout = Some(blockout);
                self.status = Some("Editable blockout created".into());
                self.error = None;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn open_blockout(&mut self, path: &Path) -> Result<(), String> {
        match load_blockout(path) {
            Ok(blockout) => {
                self.name = blockout.name.clone();
                self.document_path = path.display().to_string();
                self.export_path = path.with_extension("obj").display().to_string();
                self.blockout = Some(blockout);
                self.status = Some("Blockout reopened".into());
                self.error = None;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn add_block(&mut self) -> Result<(), String> {
        let blockout = self
            .blockout
            .as_mut()
            .ok_or_else(|| "create or open a blockout first".to_string())?;
        match blockout.add_block(self.block_min, self.block_max) {
            Ok(id) => {
                self.move_id = id;
                self.status = Some(format!("Block {id} added"));
                self.error = None;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn move_selected_block(&mut self) -> Result<(), String> {
        let blockout = self
            .blockout
            .as_mut()
            .ok_or_else(|| "create or open a blockout first".to_string())?;
        match blockout.move_block(self.move_id, self.delta) {
            Ok(()) => {
                self.status = Some(format!("Block {} moved", self.move_id));
                self.error = None;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn save_document(&mut self) -> Result<(), String> {
        let blockout = self
            .blockout
            .as_ref()
            .ok_or_else(|| "create or open a blockout first".to_string())?;
        let path = Path::new(&self.document_path);
        match save_blockout(blockout, path, path.exists()) {
            Ok(()) => {
                self.status = Some(format!("Blockout saved: {}", self.document_path));
                self.error = None;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn export_mesh(&mut self) -> Result<(), String> {
        let blockout = self
            .blockout
            .as_ref()
            .ok_or_else(|| "create or open a blockout first".to_string())?;
        match export_obj(blockout, Path::new(&self.export_path)) {
            Ok(()) => {
                self.status = Some(format!("Inspection mesh: {}", self.export_path));
                self.error = None;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                Err(error)
            }
        }
    }
}

impl eframe::App for MapStrikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MapStrike");
            ui.label("Editable blockout study; OBJ export is not a Valve map build.");
            ui.horizontal(|ui| {
                ui.label("New blockout name");
                ui.text_edit_singleline(&mut self.name);
                if ui.button("New blockout").clicked() {
                    let _ = self.new_blockout();
                }
                if ui.button("Open blockout").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Blockout JSON", &["json"])
                        .pick_file()
                    {
                        let _ = self.open_blockout(&path);
                    }
                }
            });
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::LIGHT_RED, error);
            }
            if let Some(status) = &self.status {
                ui.colored_label(egui::Color32::LIGHT_GREEN, status);
            }
            if let Some(blockout) = &self.blockout {
                ui.separator();
                ui.label(format!("{} blocks", blockout.blocks.len()));
                for block in &blockout.blocks {
                    ui.label(format!(
                        "#{} min {:?} max {:?}",
                        block.id, block.min, block.max
                    ));
                }
                ui.horizontal(|ui| {
                    ui.label("Min XYZ");
                    for value in &mut self.block_min {
                        ui.add(egui::DragValue::new(value));
                    }
                    ui.label("Max XYZ");
                    for value in &mut self.block_max {
                        ui.add(egui::DragValue::new(value));
                    }
                });
                if ui.button("Add block").clicked() {
                    let _ = self.add_block();
                }
                ui.horizontal(|ui| {
                    ui.label("Move block ID");
                    ui.add(egui::DragValue::new(&mut self.move_id));
                    ui.label("Delta XYZ");
                    for value in &mut self.delta {
                        ui.add(egui::DragValue::new(value));
                    }
                });
                if ui.button("Move block").clicked() {
                    let _ = self.move_selected_block();
                }
                ui.horizontal(|ui| {
                    ui.label("Blockout file");
                    ui.text_edit_singleline(&mut self.document_path);
                });
                if ui.button("Save blockout").clicked() {
                    let _ = self.save_document();
                }
                ui.horizontal(|ui| {
                    ui.label("Inspection OBJ");
                    ui.text_edit_singleline(&mut self.export_path);
                });
                if ui.button("Export inspection mesh").clicked() {
                    let _ = self.export_mesh();
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::MapStrikeApp;
    use std::path::Path;

    #[test]
    fn view_model_creates_moves_saves_reopens_and_exports_blockout() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut app = MapStrikeApp::default();
        app.document_path = dir.path().join("study.json").display().to_string();
        app.export_path = dir.path().join("study.obj").display().to_string();
        app.new_blockout().unwrap();
        app.add_block().unwrap();
        app.delta = [16, 0, 0];
        app.move_selected_block().unwrap();
        app.save_document().unwrap();
        let project = app.document_path.clone();
        app.open_blockout(Path::new(&project)).unwrap();
        assert_eq!(app.blockout.as_ref().unwrap().blocks[0].min, [16, 0, 0]);
        app.export_mesh().unwrap();
        assert!(Path::new(&app.export_path).is_file());
    }
}
