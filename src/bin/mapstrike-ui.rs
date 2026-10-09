use mapstrike::ui::MapStrikeApp;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "MapStrike - blockout study",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(MapStrikeApp::default()))),
    )
}
