use crate::blockout::{Block, Blockout, Spawn, Team, export_obj, load_blockout, save_blockout};
use crate::vmap::export_vmap;
use eframe::egui;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
enum PendingDocumentAction {
    New,
    Open(PathBuf),
}

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
    dirty: bool,
    pending_document_action: Option<PendingDocumentAction>,
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
            dirty: false,
            pending_document_action: None,
            logo_texture: None,
            logo_loaded: false,
        }
    }
}

impl MapStrikeApp {
    pub fn new_blockout(&mut self) -> Result<(), String> {
        if self.blockout.is_some() && self.dirty {
            let error = "save or explicitly discard the current blockout before starting another"
                .to_string();
            self.error = Some(error.clone());
            return Err(error);
        }
        self.create_blockout_replacing()
    }

    fn create_blockout_replacing(&mut self) -> Result<(), String> {
        match Blockout::new(&self.name) {
            Ok(blockout) => {
                if self.blockout.is_some() {
                    self.document_path.clear();
                    self.export_path.clear();
                    self.vmap_path.clear();
                }
                self.blockout = Some(blockout);
                self.move_id = 1;
                self.dirty = true;
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
        if self.blockout.is_some() && self.dirty {
            let error = "save or explicitly discard the current blockout before opening another"
                .to_string();
            self.error = Some(error.clone());
            return Err(error);
        }
        self.open_blockout_replacing(path)
    }

    fn open_blockout_replacing(&mut self, path: &Path) -> Result<(), String> {
        match load_blockout(path) {
            Ok(blockout) => {
                self.name = blockout.name.clone();
                self.move_id = blockout.blocks.first().map_or(1, |block| block.id);
                self.document_path = path.display().to_string();
                self.export_path = path.with_extension("obj").display().to_string();
                self.vmap_path = path.with_extension("vmap").display().to_string();
                self.blockout = Some(blockout);
                self.dirty = false;
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

    fn request_new_blockout(&mut self) {
        if self.blockout.is_some() {
            self.pending_document_action = Some(PendingDocumentAction::New);
        } else {
            let _ = self.new_blockout();
        }
    }

    fn request_open_blockout(&mut self, path: PathBuf) {
        if self.blockout.is_some() && self.dirty {
            self.pending_document_action = Some(PendingDocumentAction::Open(path));
        } else {
            let _ = self.open_blockout(&path);
        }
    }

    fn resolve_pending_document_action(&mut self, discard: bool) {
        let Some(action) = self.pending_document_action.take() else {
            return;
        };
        if !discard {
            return;
        }
        match action {
            PendingDocumentAction::New => {
                let _ = self.create_blockout_replacing();
            }
            PendingDocumentAction::Open(path) => {
                let _ = self.open_blockout_replacing(&path);
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
                self.dirty = true;
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
                self.dirty = true;
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
                self.dirty = true;
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
                self.dirty = true;
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
                self.dirty = false;
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

    fn save_from_ui(&mut self) {
        if self.document_path.trim().is_empty() {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("Blockout JSON", &["json"])
                .set_file_name("blockout.json")
                .save_file()
            else {
                return;
            };
            self.document_path = path.display().to_string();
        }
        let _ = self.save_document();
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

    fn show_controls(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.heading("Build the blockout");
        ui.small("Work in X/Y from above. Z values control height; select a block in the plan to move it.");
        ui.add_space(12.0);
        ui.label("New blockout name");
        ui.add(egui::TextEdit::singleline(&mut self.name).desired_width(280.0));
        if ui.button("Create new blockout").clicked() {
            self.request_new_blockout();
        }
        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::LIGHT_RED, error);
        } else if let Some(status) = &self.status {
            ui.colored_label(egui::Color32::LIGHT_GREEN, status);
        }

        ui.add_space(16.0);
        ui.separator();
        ui.heading("Room");
        ui.small("Add a floor and four walls around an interior.");
        coordinate_row(ui, "Interior minimum", &mut self.room_origin);
        coordinate_row(ui, "Interior size", &mut self.room_size);
        ui.horizontal(|ui| {
            ui.label("Wall thickness");
            ui.add(egui::DragValue::new(&mut self.room_thickness).range(1..=1024));
        });
        if ui.button("Add floor and walls").clicked() {
            let _ = self.add_room();
        }

        ui.add_space(16.0);
        ui.separator();
        ui.heading("Team spawns");
        egui::ComboBox::from_label("Team")
            .selected_text(match self.spawn_team {
                Team::Terrorists => "Terrorists",
                Team::CounterTerrorists => "Counter-Terrorists",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.spawn_team, Team::Terrorists, "Terrorists");
                ui.selectable_value(
                    &mut self.spawn_team,
                    Team::CounterTerrorists,
                    "Counter-Terrorists",
                );
            });
        coordinate_row(ui, "Origin", &mut self.spawn_origin);
        ui.horizontal(|ui| {
            ui.label("Yaw");
            ui.add(
                egui::DragValue::new(&mut self.spawn_yaw)
                    .range(0..=359)
                    .suffix("°"),
            );
        });
        if ui.button("Add team spawn").clicked() {
            let _ = self.add_spawn();
        }

        ui.add_space(16.0);
        ui.separator();
        ui.heading("Blocks");
        coordinate_row(ui, "Minimum", &mut self.block_min);
        coordinate_row(ui, "Maximum", &mut self.block_max);
        if ui.button("Add block").clicked() {
            let _ = self.add_block();
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Selected ID");
            ui.add(egui::DragValue::new(&mut self.move_id).range(1..=u32::MAX));
        });
        coordinate_row(ui, "Move by", &mut self.delta);
        if ui.button("Move selected block").clicked() {
            let _ = self.move_selected_block();
        }

        ui.add_space(16.0);
        ui.separator();
        ui.heading("Save and export");
        ui.label("Editable blockout JSON");
        ui.add(egui::TextEdit::singleline(&mut self.document_path).desired_width(280.0));
        if ui.button("Save blockout JSON").clicked() {
            self.save_from_ui();
        }
        ui.add_space(8.0);
        ui.label("Inspection OBJ");
        ui.add(egui::TextEdit::singleline(&mut self.export_path).desired_width(280.0));
        if ui.button("Export inspection mesh").clicked() {
            let _ = self.export_mesh();
        }
        ui.add_space(8.0);
        ui.label("Experimental CS2 VMAP");
        ui.add(egui::TextEdit::singleline(&mut self.vmap_path).desired_width(280.0));
        ui.small("Requires both T and CT spawns. Validate in Hammer and CS2 before use.");
        if ui.button("Export new VMAP source").clicked() {
            let _ = self.export_cs2_source();
        }
        ui.add_space(12.0);
    }

    fn show_plan(&mut self, ui: &mut egui::Ui) {
        let Some(blockout) = &self.blockout else {
            return;
        };
        ui.add_space(8.0);
        ui.heading("Top-down plan");
        ui.label(format!(
            "{} blocks  ·  {} team spawns",
            blockout.blocks.len(),
            blockout.spawns.len()
        ));
        ui.small("Click a block to select it. This is an X/Y planning view, not a Hammer or in-game preview.");
        ui.add_space(10.0);
        let size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(size.x.max(240.0), size.y.max(240.0)),
            egui::Sense::click(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(22, 27, 34));
        if blockout.blocks.is_empty() && blockout.spawns.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Add a room or a block to see the plan",
                egui::FontId::proportional(16.0),
                egui::Color32::from_rgb(205, 215, 222),
            );
            return;
        }
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        for block in &blockout.blocks {
            min_x = min_x.min(block.min[0]);
            min_y = min_y.min(block.min[1]);
            max_x = max_x.max(block.max[0]);
            max_y = max_y.max(block.max[1]);
        }
        for spawn in &blockout.spawns {
            min_x = min_x.min(spawn.origin[0].saturating_sub(32));
            min_y = min_y.min(spawn.origin[1].saturating_sub(32));
            max_x = max_x.max(spawn.origin[0].saturating_add(32));
            max_y = max_y.max(spawn.origin[1].saturating_add(32));
        }
        let span_x = (i64::from(max_x) - i64::from(min_x)).max(1) as f32;
        let span_y = (i64::from(max_y) - i64::from(min_y)).max(1) as f32;
        let inset = rect.shrink(36.0);
        let scale = (inset.width() / span_x).min(inset.height() / span_y);
        let origin = egui::pos2(
            inset.center().x - span_x * scale * 0.5,
            inset.center().y + span_y * scale * 0.5,
        );
        let point = |x: i32, y: i32| {
            egui::pos2(
                origin.x + (i64::from(x) - i64::from(min_x)) as f32 * scale,
                origin.y - (i64::from(y) - i64::from(min_y)) as f32 * scale,
            )
        };
        let mut picked = None;
        for block in &blockout.blocks {
            let area = egui::Rect::from_two_pos(
                point(block.min[0], block.min[1]),
                point(block.max[0], block.max[1]),
            );
            let selected = block.id == self.move_id;
            painter.rect_filled(
                area,
                1.0,
                if selected {
                    egui::Color32::from_rgb(111, 170, 193)
                } else {
                    egui::Color32::from_rgb(70, 94, 108)
                },
            );
            if area.width() > 34.0 && area.height() > 20.0 {
                painter.text(
                    area.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("#{}", block.id),
                    egui::FontId::proportional(13.0),
                    egui::Color32::WHITE,
                );
            }
            if response.clicked()
                && response
                    .interact_pointer_pos()
                    .is_some_and(|pos| area.contains(pos))
            {
                picked = Some(block.id);
            }
        }
        for spawn in &blockout.spawns {
            let pos = point(spawn.origin[0], spawn.origin[1]);
            let (label, color) = match spawn.team {
                Team::Terrorists => ("T", egui::Color32::from_rgb(235, 173, 92)),
                Team::CounterTerrorists => ("CT", egui::Color32::from_rgb(107, 169, 223)),
            };
            painter.circle_filled(pos, 12.0, color);
            painter.text(
                pos,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.0),
                egui::Color32::BLACK,
            );
        }
        if let Some(id) = picked {
            self.move_id = id;
        }
    }

    fn show_inventory(&mut self, ui: &mut egui::Ui) {
        let Some(blockout) = &self.blockout else {
            return;
        };
        let mut picked = None;
        ui.add_space(8.0);
        ui.heading("Blockout contents");
        ui.small("Exact X/Y/Z values for the plan. Select a block here or in the plan to move it.");
        ui.add_space(10.0);
        ui.heading(format!("Blocks ({})", blockout.blocks.len()));
        if blockout.blocks.is_empty() {
            ui.label("No blocks yet. Add a room or block from the controls.");
        }
        for block in &blockout.blocks {
            ui.group(|ui| {
                if ui
                    .selectable_label(self.move_id == block.id, format!("Block #{}", block.id))
                    .clicked()
                {
                    picked = Some(block.id);
                }
                for line in block_detail_lines(block) {
                    ui.monospace(line);
                }
            });
        }
        ui.add_space(10.0);
        ui.heading(format!("Team spawns ({})", blockout.spawns.len()));
        if blockout.spawns.is_empty() {
            ui.label("No team spawns yet.");
        }
        for spawn in &blockout.spawns {
            ui.group(|ui| {
                ui.label(format!("Spawn #{} · {:?}", spawn.id, spawn.team));
                for line in spawn_detail_lines(spawn) {
                    ui.monospace(line);
                }
            });
        }
        if let Some(id) = picked {
            self.move_id = id;
        }
    }
}

fn block_detail_lines(block: &Block) -> [String; 2] {
    let [x0, y0, z0] = block.min;
    let [x1, y1, z1] = block.max;
    [
        format!("Min X:{x0} Y:{y0} Z:{z0}"),
        format!("Max X:{x1} Y:{y1} Z:{z1}"),
    ]
}

fn spawn_detail_lines(spawn: &Spawn) -> [String; 2] {
    let [x, y, z] = spawn.origin;
    [format!("X:{x} Y:{y} Z:{z}"), format!("Yaw {}°", spawn.yaw)]
}

fn coordinate_row(ui: &mut egui::Ui, label: &str, values: &mut [i32; 3]) {
    ui.label(label);
    ui.horizontal(|ui| {
        for (axis, value) in ["X", "Y", "Z"].into_iter().zip(values.iter_mut()) {
            ui.label(axis);
            ui.add_sized([55.0, 22.0], egui::DragValue::new(value));
        }
    });
}

impl eframe::App for MapStrikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        ctx.set_style(style);
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
                    ui.label("Shape a CS2 blockout and prepare it for Hammer.");
                    ui.small("A PWDS project");
                    ui.add_space(22.0);
                    ui.horizontal(|ui| {
                        ui.label("New blockout");
                        ui.text_edit_singleline(&mut self.name);
                        if ui.button("Create").clicked() {
                            self.request_new_blockout();
                        }
                    });
                    if ui.button("Open existing blockout").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Blockout JSON", &["json"])
                            .pick_file()
                        {
                            self.request_open_blockout(path);
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
        egui::TopBottomPanel::top("workspace_header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("MapStrike");
                ui.separator();
                ui.label(&self.name);
                if self.dirty {
                    ui.colored_label(egui::Color32::from_rgb(236, 177, 104), "Unsaved changes");
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Open blockout").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Blockout JSON", &["json"])
                            .pick_file()
                        {
                            self.request_open_blockout(path);
                        }
                    }
                    if ui.button("Save blockout").clicked() {
                        self.save_from_ui();
                    }
                });
            });
        });
        egui::SidePanel::left("blockout_controls")
            .resizable(true)
            .default_width(320.0)
            .min_width(275.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.show_controls(ui));
            });
        egui::SidePanel::right("blockout_inventory")
            .resizable(true)
            .default_width(270.0)
            .min_width(230.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.show_inventory(ui));
            });
        egui::CentralPanel::default().show(ctx, |ui| self.show_plan(ui));
        if let Some(action) = &self.pending_document_action {
            let (title, prompt, confirm_label) = match action {
                PendingDocumentAction::New => (
                    "Start a new blockout",
                    "Replace the current in-memory blockout?",
                    "Discard edits and start new",
                ),
                PendingDocumentAction::Open(_) => (
                    "Open another blockout",
                    "Open the selected blockout instead of the current one?",
                    "Discard edits and open",
                ),
            };
            let mut decision = None;
            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(prompt);
                    ui.label("Saved files stay on disk. Unsaved edits will be lost.");
                    ui.horizontal(|ui| {
                        if ui.button("Keep editing").clicked() {
                            decision = Some(false);
                        }
                        if ui.button(confirm_label).clicked() {
                            decision = Some(true);
                        }
                    });
                });
            if let Some(discard) = decision {
                self.resolve_pending_document_action(discard);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MapStrikeApp, Team, block_detail_lines, spawn_detail_lines};
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
    fn starting_another_blockout_does_not_reuse_previous_output_paths() {
        let mut app = MapStrikeApp::default();
        app.new_blockout().unwrap();
        let previous = app.blockout.clone();
        app.document_path = "previous.json".into();
        app.export_path = "previous.obj".into();
        app.vmap_path = "previous.vmap".into();
        app.name = "Next study".into();
        app.request_new_blockout();
        app.resolve_pending_document_action(false);
        assert_eq!(app.blockout, previous);
        assert_eq!(app.document_path, "previous.json");
        assert!(app.new_blockout().is_err());
        app.request_new_blockout();
        app.resolve_pending_document_action(true);
        assert_eq!(app.blockout.as_ref().unwrap().name, "Next study");
        assert!(app.document_path.is_empty());
        assert!(app.export_path.is_empty());
        assert!(app.vmap_path.is_empty());
    }

    #[test]
    fn opening_another_blockout_needs_discard_and_failed_open_preserves_current() {
        let dir = tempfile::TempDir::new().unwrap();
        let target = dir.path().join("target.json");
        let mut saved = MapStrikeApp::default();
        saved.name = "Saved study".into();
        saved.document_path = target.display().to_string();
        saved.new_blockout().unwrap();
        saved.save_document().unwrap();

        let mut app = MapStrikeApp::default();
        app.new_blockout().unwrap();
        app.add_block().unwrap();
        let current = app.blockout.clone();
        app.request_open_blockout(target.clone());
        app.resolve_pending_document_action(false);
        assert_eq!(app.blockout, current);
        assert!(app.dirty);
        assert!(app.open_blockout(&target).is_err());
        assert_eq!(app.blockout, current);

        app.request_open_blockout(dir.path().join("missing.json"));
        app.resolve_pending_document_action(true);
        assert_eq!(app.blockout, current);
        assert!(app.dirty);
        assert!(app.error.as_deref().unwrap().contains("missing"));

        app.request_open_blockout(target.clone());
        app.resolve_pending_document_action(true);
        assert_eq!(app.blockout.as_ref().unwrap().name, "Saved study");
        assert_eq!(app.document_path, target.display().to_string());
        assert!(!app.dirty);
    }

    #[test]
    fn reopened_inventory_retains_block_height_and_spawn_heading() {
        let dir = tempfile::TempDir::new().unwrap();
        let document = dir.path().join("details.json");
        let mut app = MapStrikeApp::default();
        app.document_path = document.display().to_string();
        app.new_blockout().unwrap();
        app.block_min = [-12, 34, 56];
        app.block_max = [78, 90, 123];
        app.add_block().unwrap();
        app.spawn_team = Team::CounterTerrorists;
        app.spawn_origin = [201, -45, 17];
        app.spawn_yaw = 270;
        app.add_spawn().unwrap();
        app.save_document().unwrap();

        app.move_id = 999;
        app.open_blockout(&document).unwrap();
        let reopened = app.blockout.as_ref().unwrap();
        assert_eq!(app.move_id, reopened.blocks[0].id);
        assert_eq!(
            block_detail_lines(&reopened.blocks[0]),
            ["Min X:-12 Y:34 Z:56", "Max X:78 Y:90 Z:123"]
        );
        assert_eq!(reopened.spawns[0].team, Team::CounterTerrorists);
        assert_eq!(
            spawn_detail_lines(&reopened.spawns[0]),
            ["X:201 Y:-45 Z:17", "Yaw 270°"]
        );
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
