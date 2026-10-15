//! Capture one frame from the actual native blockout UI for documentation.
use eframe::egui;
use mapstrike::ui::MapStrikeApp;
use std::ffi::OsString;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

fn capture_paths(args: &[OsString]) -> Result<(PathBuf, PathBuf), String> {
    match args {
        [document, output] => Ok((PathBuf::from(document), PathBuf::from(output))),
        _ => Err("usage: mapstrike-capture <blockout.json> <new.png>".into()),
    }
}

struct Capture {
    app: MapStrikeApp,
    output: PathBuf,
    frames: usize,
}

impl eframe::App for Capture {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.update(ctx, frame);
        for event in ctx.input(|input| input.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                if let Err(error) = save_png(&self.output, &image) {
                    eprintln!("capture failed: {error}");
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        self.frames += 1;
        if self.frames == 2 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }
}

fn save_png(path: &Path, image: &egui::ColorImage) -> Result<(), String> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    let mut encoder = png::Encoder::new(
        BufWriter::new(file),
        image.size[0] as u32,
        image.size[1] as u32,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    let rgba: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|pixel| pixel.to_array())
        .collect();
    writer
        .write_image_data(&rgba)
        .map_err(|error| error.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let (document, output) = capture_paths(&args)?;
    if output.exists() {
        return Err("capture output must be a new file".into());
    }
    let mut app = MapStrikeApp::default();
    app.open_blockout(&document)?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([650.0, 370.0]),
        ..Default::default()
    };
    let capture_output = output.clone();
    eframe::run_native(
        "MapStrike renderer capture",
        options,
        Box::new(move |_| {
            Ok(Box::new(Capture {
                app,
                output: capture_output,
                frames: 0,
            }))
        }),
    )?;
    if !output.is_file() || output.metadata()?.len() == 0 {
        return Err("renderer did not write a PNG".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn accepts_exactly_one_document_and_one_new_image_path() {
        let args = vec![OsString::from("study.json"), OsString::from("view.png")];
        assert_eq!(
            capture_paths(&args).unwrap(),
            (PathBuf::from("study.json"), PathBuf::from("view.png"))
        );
        assert!(capture_paths(&args[..1]).is_err());
        assert!(
            capture_paths(&[args[0].clone(), args[1].clone(), OsString::from("extra")]).is_err()
        );
    }

    #[test]
    fn failed_encoding_does_not_publish_partial_png() {
        let dir = tempfile::TempDir::new().unwrap();
        let output = dir.path().join("failed.png");
        let invalid_image = egui::ColorImage {
            size: [1, 1],
            pixels: Vec::new(),
        };
        assert!(save_png(&output, &invalid_image).is_err());
        assert!(!output.exists(), "failed encoding left a partial PNG");
    }
}
