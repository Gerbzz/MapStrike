//! Capture one frame from the actual native blockout UI for documentation.
use eframe::egui;
use mapstrike::ui::MapStrikeApp;
use std::ffi::OsString;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::time::{Duration, Instant};

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
    started: Instant,
    outcome: Option<Sender<Result<(), String>>>,
}

impl Capture {
    fn finish(&mut self, ctx: &egui::Context, result: Result<(), String>) {
        if let Some(outcome) = self.outcome.take() {
            let _ = outcome.send(result);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for Capture {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.outcome.is_none() {
            return;
        }
        self.app.update(ctx, frame);
        for event in ctx.input(|input| input.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                let result = save_png(&self.output, &image);
                self.finish(ctx, result);
                return;
            }
        }
        self.frames += 1;
        if self.frames == 2 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        if self.started.elapsed() >= Duration::from_secs(15) {
            self.finish(ctx, Err("timed out waiting for renderer screenshot".into()));
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

fn save_png(path: &Path, image: &egui::ColorImage) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    encode_png(BufWriter::new(pending.as_file_mut()), image)?;
    pending
        .as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    pending
        .persist_noclobber(path)
        .map_err(|error| error.error.to_string())?;
    Ok(())
}

fn encode_png<W: Write>(sink: W, image: &egui::ColorImage) -> Result<(), String> {
    let mut encoder = png::Encoder::new(sink, image.size[0] as u32, image.size[1] as u32);
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
        .map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())
}

fn capture_completed(outcome: Result<(), String>, output: &Path) -> Result<(), String> {
    outcome?;
    if !output.is_file() || output.metadata().map_err(|error| error.to_string())?.len() == 0 {
        return Err("renderer did not write a PNG".into());
    }
    Ok(())
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
    let (outcome_sender, outcome_receiver) = mpsc::channel();
    eframe::run_native(
        "MapStrike renderer capture",
        options,
        Box::new(move |_| {
            Ok(Box::new(Capture {
                app,
                output: capture_output,
                frames: 0,
                started: Instant::now(),
                outcome: Some(outcome_sender),
            }))
        }),
    )?;
    let outcome = outcome_receiver
        .recv()
        .map_err(|_| "capture ended before screenshot completed")?;
    capture_completed(outcome, &output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::io;
    use std::path::PathBuf;

    struct FlushFailure;

    impl Write for FlushFailure {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("deliberate final flush failure"))
        }
    }

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

    #[test]
    fn final_png_flush_failure_is_reported() {
        let image = egui::ColorImage::new([1, 1], egui::Color32::WHITE);
        assert!(encode_png(FlushFailure, &image).is_err());
    }

    #[test]
    fn capture_error_overrides_a_nonempty_output() {
        let dir = tempfile::TempDir::new().unwrap();
        let output = dir.path().join("partial.png");
        std::fs::write(&output, b"partial").unwrap();
        assert_eq!(
            capture_completed(Err("save failed".into()), &output).unwrap_err(),
            "save failed"
        );
    }
}
