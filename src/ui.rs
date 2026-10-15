use crate::blockout::{Blockout, Team, export_obj, load_blockout, save_blockout};
use crate::vmap::export_vmap;
use eframe::egui;
use std::path::Path;

pub struct MapStrikeApp {
    pub blockout: Option<Blockout>,
    pub name: String,
    pub document_path: String,
    pub export_path: String,
    pub vmap_path: String,
    pub block_min: [i32; 3],
    pub block_max: [i32; 3],
    pub move_id: u32,
    pub delta: [i32; 3],
    pub room_origin: [i32; 3],
    pub room_size: [i32; 3],
    pub room_thickness: i32,
    pub spawn_team: Team,
    pub spawn_origin: [i32; 3],
    pub spawn_yaw: u16,
    pub status: Option<String>,
    pub error: Option<String>,
    logo_texture: Option<egui::TextureHandle>,
    logo_loaded: bool,
}

impl Default for MapStrikeApp {
    fn default() -> Self {
        Self {
            blockout: None,
            name: "New blockout".into(),
            document_path: String::new(),
            export_path: String::new(),
            vmap_path: String::new(),
            block_min: [0, 0, 0],
            block_max: [128, 128, 64],
            move_id: 1,
            delta: [0, 0, 0],
            room_origin: [-512, -512, 0],
            room_size: [1024, 1024, 256],
            room_thickness: 16,
            spawn_team: Team::Terrorists,
            spawn_origin: [-256, 0, 16],
            spawn_yaw: 0,
            status: None,
            error: None,
            logo_texture: None,
            logo_loaded: false,
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
                self.vmap_path = path.with_extension("vmap").display().to_string();
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

    pub fn add_room(&mut self) -> Result<(), String> {
        let blockout = self
            .blockout
            .as_mut()
            .ok_or_else(|| "create or open a blockout first".to_string())?;
        match blockout.add_room(self.room_origin, self.room_size, self.room_thickness) {
            Ok(ids) => {
                self.status = Some(format!(
                    "Room floor and walls added as blocks {}–{}",
                    ids[0], ids[4]
                ));
                self.error = None;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn add_spawn(&mut self) -> Result<(), String> {
        let blockout = self
            .blockout
            .as_mut()
            .ok_or_else(|| "create or open a blockout first".to_string())?;
        match blockout.add_spawn(self.spawn_team, self.spawn_origin, self.spawn_yaw) {
            Ok(id) => {
                self.status = Some(format!("{:?} spawn {id} added", self.spawn_team));
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

    pub fn export_cs2_source(&mut self) -> Result<(), String> {
        let blockout = self
            .blockout
            .as_ref()
            .ok_or_else(|| "create or open a blockout first".to_string())?;
        match export_vmap(blockout, Path::new(&self.vmap_path)) {
            Ok(()) => {
                self.status = Some(format!(
                    "Experimental VMAP source written; Hammer validation pending: {}",
                    self.vmap_path
                ));
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
        if !self.logo_loaded {
            self.logo_loaded = true;
            if let Ok(png) =
                image::load_from_memory(include_bytes!("../assets/branding/MapStrike.png"))
            {
                let rgba = png.to_rgba8();
                let color = egui::ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as usize, rgba.height() as usize],
                    rgba.as_raw(),
                );
                self.logo_texture = Some(ctx.load_texture(
                    "approved-mapstrike-logo",
                    color,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        if self.blockout.is_none() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);
                    if let Some(logo) = &self.logo_texture {
                        ui.image((logo.id(), egui::vec2(128.0, 128.0)));
                    }
                    ui.heading("MapStrike");
                    ui.label("Shape a space, then test it in Counter-Strike 2.");
                    ui.small("A PWDS project");
                    ui.add_space(22.0);
                    ui.horizontal(|ui| {
                        ui.label("New blockout");
                        ui.text_edit_singleline(&mut self.name);
                        if ui.button("Create").clicked() {
                            let _ = self.new_blockout();
                        }
                    });
                    if ui.button("Open existing blockout").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Blockout JSON", &["json"])
                            .pick_file()
                        {
                            let _ = self.open_blockout(&path);
                        }
                    }
                    if let Some(error) = &self.error {
                        ui.colored_label(egui::Color32::LIGHT_RED, error);
                    }
                    ui.small("VMAP export is experimental until Hammer and in-game checks pass.");
                });
            });
            return;
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Blockout editor");
                ui.label(
                    "Editable room and team-spawn study; OBJ export is not a Valve map build.",
                );
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
                    ui.label(format!(
                        "{} blocks · {} team spawns",
                        blockout.blocks.len(),
                        blockout.spawns.len()
                    ));
                    for block in &blockout.blocks {
                        ui.label(format!(
                            "#{} min {:?} max {:?}",
                            block.id, block.min, block.max
                        ));
                    }
                    for spawn in &blockout.spawns {
                        ui.label(format!(
                            "Spawn #{} {:?} at {:?}, yaw {}°",
                            spawn.id, spawn.team, spawn.origin, spawn.yaw
                        ));
                    }
                    ui.separator();
                    ui.label("Open room: inside minimum, interior size, wall/floor thickness");
                    ui.horizontal(|ui| {
                        ui.label("Min XYZ");
                        for value in &mut self.room_origin {
                            ui.add(egui::DragValue::new(value));
                        }
                        ui.label("Size XYZ");
                        for value in &mut self.room_size {
                            ui.add(egui::DragValue::new(value));
                        }
                        ui.label("Thickness");
                        ui.add(egui::DragValue::new(&mut self.room_thickness));
                    });
                    if ui.button("Add floor and four walls").clicked() {
                        let _ = self.add_room();
                    }
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_label("Team")
                            .selected_text(match self.spawn_team {
                                Team::Terrorists => "Terrorists",
                                Team::CounterTerrorists => "Counter-Terrorists",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.spawn_team,
                                    Team::Terrorists,
                                    "Terrorists",
                                );
                                ui.selectable_value(
                                    &mut self.spawn_team,
                                    Team::CounterTerrorists,
                                    "Counter-Terrorists",
                                );
                            });
                        ui.label("Origin XYZ");
                        for value in &mut self.spawn_origin {
                            ui.add(egui::DragValue::new(value));
                        }
                        ui.label("Yaw");
                        ui.add(egui::DragValue::new(&mut self.spawn_yaw).range(0..=359));
                    });
                    if ui.button("Add team spawn").clicked() {
                        let _ = self.add_spawn();
                    }
                    ui.separator();
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
                    ui.horizontal(|ui| {
                        ui.label("Experimental CS2 VMAP");
                        ui.text_edit_singleline(&mut self.vmap_path);
                    });
                    ui.label("Requires T and CT spawns. Hammer open, compile, and CS2 play checks are pending.");
                    if ui.button("Export new VMAP source").clicked() {
                        let _ = self.export_cs2_source();
                    }
                }
            });
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

    #[test]
    fn view_model_adds_room_and_team_spawns() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut app = MapStrikeApp::default();
        app.document_path = dir.path().join("room.json").display().to_string();
        app.vmap_path = dir.path().join("room.vmap").display().to_string();
        app.new_blockout().unwrap();
        app.add_room().unwrap();
        app.add_spawn().unwrap();
        app.spawn_team = super::Team::CounterTerrorists;
        app.spawn_origin = [256, 0, 16];
        app.spawn_yaw = 180;
        app.add_spawn().unwrap();
        app.save_document().unwrap();
        let document = app.document_path.clone();
        app.open_blockout(Path::new(&document)).unwrap();
        let blockout = app.blockout.as_ref().unwrap();
        assert_eq!(blockout.blocks.len(), 5);
        assert_eq!(blockout.spawns.len(), 2);
        app.export_cs2_source().unwrap();
        assert!(Path::new(&app.vmap_path).is_file());
        assert!(app.export_cs2_source().is_err());
    }
}
