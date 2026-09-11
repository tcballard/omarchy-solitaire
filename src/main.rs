use omarchy_solitaire::{
    app::SolitaireApp,
    storage::{self, SessionLock},
};
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dir = storage::state_dir();
    let mut screenshot = None;
    let (mut width, mut height) = (1120., 800.);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--version" => {
                println!("Omarchy Solitaire {} (Rust)", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" | "-h" => {
                println!("Omarchy Solitaire\n\n--state-dir PATH  Isolate saves\n--screenshot PNG  Capture the native window and exit\n--width N --height N  Window size (minimum 800 × 600)\n--version");
                return Ok(());
            }
            "--state-dir" => dir = PathBuf::from(args.next().ok_or("Missing state directory")?),
            "--screenshot" => {
                screenshot = Some(PathBuf::from(args.next().ok_or("Missing screenshot path")?))
            }
            "--width" => {
                width = args
                    .next()
                    .ok_or("Missing width")?
                    .parse::<f32>()?
                    .clamp(800., 4096.)
            }
            "--height" => {
                height = args
                    .next()
                    .ok_or("Missing height")?
                    .parse::<f32>()?
                    .clamp(600., 4096.)
            }
            _ => return Err(format!("Unknown option: {arg}").into()),
        }
    }
    let _lock = SessionLock::acquire(&dir).map_err(|e| {
        eprintln!("Omarchy Solitaire: {e}");
        e
    })?;
    let icon = egui_extras::image::load_svg_bytes_with_size(
        include_bytes!("../assets/solitaire.svg"),
        Some(eframe::egui::load::SizeHint::Size(128, 128)),
    )?;
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_min_inner_size([800., 600.])
            .with_app_id("io.github.tcballard.omarchy-solitaire")
            .with_icon(eframe::egui::IconData {
                rgba: icon.pixels.iter().flat_map(|p| p.to_array()).collect(),
                width: icon.width() as u32,
                height: icon.height() as u32,
            }),
        ..Default::default()
    };
    eframe::run_native(
        "Omarchy Solitaire",
        options,
        Box::new(move |cc| Ok(Box::new(SolitaireApp::new(&cc.egui_ctx, dir, screenshot)))),
    )?;
    Ok(())
}
