use crate::storage::{self, read_bounded};
use eframe::egui::Color32;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Clone, Debug)]
pub struct Palette {
    pub table: Color32,
    pub surface: Color32,
    pub text: Color32,
    pub accent: Color32,
    pub line: Color32,
    pub back: Color32,
    pub pattern: Color32,
}
pub const FACE: Color32 = Color32::from_rgb(250, 247, 239);
pub const INK: Color32 = Color32::from_rgb(32, 40, 38);
pub const RED: Color32 = Color32::from_rgb(171, 38, 61);
pub fn hex(s: &str) -> Option<Color32> {
    if s.len() != 7 || !s.starts_with('#') || !s[1..].bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some(Color32::from_rgb(
        u8::from_str_radix(&s[1..3], 16).ok()?,
        u8::from_str_radix(&s[3..5], 16).ok()?,
        u8::from_str_radix(&s[5..7], 16).ok()?,
    ))
}
pub fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgb(
        (a.r() as f32 * (1. - t) + b.r() as f32 * t).round() as u8,
        (a.g() as f32 * (1. - t) + b.g() as f32 * t).round() as u8,
        (a.b() as f32 * (1. - t) + b.b() as f32 * t).round() as u8,
    )
}
fn luminance(c: Color32) -> f32 {
    let linear = |v: u8| {
        let x = v as f32 / 255.;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(c.r()) + 0.7152 * linear(c.g()) + 0.0722 * linear(c.b())
}
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}
fn readable(a: Color32, b: Color32) -> Color32 {
    if contrast(a, b) >= 4.5 {
        a
    } else if contrast(Color32::BLACK, b) > contrast(Color32::WHITE, b) {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}
impl Default for Palette {
    fn default() -> Self {
        Self::new(
            hex("#182622").unwrap(),
            hex("#e6eadb").unwrap(),
            hex("#b7ce91").unwrap(),
        )
    }
}
impl Palette {
    pub fn new(table: Color32, text: Color32, accent: Color32) -> Self {
        let mut text = readable(text, table);
        let mut accent = readable(accent, table);
        let mut surface = blend(table, text, 0.06);
        text = readable(text, surface);
        accent = readable(accent, surface);
        if contrast(text, table) < 4.5 || contrast(accent, table) < 4.5 {
            surface = table;
            text = readable(text, table);
            accent = readable(accent, table);
        }
        Self {
            table,
            surface,
            text,
            accent,
            line: blend(table, text, 0.22),
            back: blend(table, accent, 0.16),
            pattern: blend(table, accent, 0.37),
        }
    }
    pub fn snapshot(&self) -> BTreeMap<String, String> {
        [
            ("table", self.table),
            ("text", self.text),
            ("accent", self.accent),
        ]
        .into_iter()
        .map(|(k, c)| {
            (
                k.into(),
                format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b()),
            )
        })
        .collect()
    }
    pub fn restore(raw: &BTreeMap<String, String>) -> Self {
        let defaults = Self::default();
        Self::new(
            raw.get("table")
                .and_then(|s| hex(s))
                .unwrap_or(defaults.table),
            raw.get("text")
                .and_then(|s| hex(s))
                .unwrap_or(defaults.text),
            raw.get("accent")
                .and_then(|s| hex(s))
                .unwrap_or(defaults.accent),
        )
    }
}
#[derive(Clone)]
pub struct Art {
    pub bytes: Vec<u8>,
    pub suffix: String,
}
pub fn validate_art(path: &Path) -> Result<Art, String> {
    let bytes = read_bounded(path, 2_000_000)?;
    let suffix = path
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if suffix == "svg" {
        let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
        if text.to_ascii_uppercase().contains("<!DOCTYPE")
            || text.to_ascii_uppercase().contains("<!ENTITY")
        {
            return Err("SVG entities are not supported.".into());
        }
        let doc = roxmltree::Document::parse(text).map_err(|e| e.to_string())?;
        if doc.root_element().tag_name().name() != "svg" {
            return Err("Choose an SVG image.".into());
        }
        for n in doc.descendants().filter(|n| n.is_element()) {
            if ![
                "svg",
                "g",
                "path",
                "rect",
                "circle",
                "ellipse",
                "line",
                "polyline",
                "polygon",
                "defs",
                "linearGradient",
                "radialGradient",
                "stop",
                "clipPath",
                "title",
                "desc",
            ]
            .contains(&n.tag_name().name())
            {
                return Err("Use static SVG paths and shapes; outline text first.".into());
            }
            for a in n.attributes() {
                let name = a.name().to_ascii_lowercase();
                let val = a.value().to_ascii_lowercase();
                if name.starts_with("on") || ["href", "src", "style"].contains(&name.as_str()) {
                    return Err("Linked images, styles and scripts are not supported.".into());
                }
                if val.contains("url(")
                    && !(val.starts_with("url(#")
                        && val.ends_with(')')
                        && val[5..val.len() - 1]
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'))
                {
                    return Err("SVG references must remain inside the image.".into());
                }
            }
        }
        // Parse and rasterize once now to reject invalid dimensions and renderer errors.
        egui_extras::image::load_svg_bytes_with_size(
            &bytes,
            Some(eframe::egui::load::SizeHint::Size(500, 700)),
        )
        .map_err(|e| e.to_string())?;
    } else if suffix == "png" {
        let reader =
            image::ImageReader::with_format(std::io::Cursor::new(&bytes), image::ImageFormat::Png);
        let (w, h) = reader.into_dimensions().map_err(|e| e.to_string())?;
        if w == 0 || h == 0 || w > 4096 || h > 4096 {
            return Err("PNG dimensions must be at most 4096 pixels per side.".into());
        }
        image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
    } else {
        return Err("Choose an SVG or PNG smaller than 2 MB.".into());
    }
    Ok(Art { bytes, suffix })
}
#[derive(Default)]
pub struct Theme {
    pub palette: Palette,
    pub name: String,
    pub art: Option<Art>,
    pub error: String,
    signature: Vec<(PathBuf, u64, SystemTime, u64)>,
}
impl Theme {
    pub fn candidates() -> Vec<PathBuf> {
        vec![
            storage::xdg("XDG_STATE_HOME", ".local/state").join("omarchy/current/theme"),
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join(".local/state/omarchy/current/theme"),
            storage::xdg("XDG_CONFIG_HOME", ".config").join("omarchy/current/theme"),
        ]
    }
    pub fn refresh(&mut self) -> bool {
        self.refresh_at(&Self::candidates())
    }
    pub fn refresh_at(&mut self, dirs: &[PathBuf]) -> bool {
        use std::os::unix::fs::MetadataExt;
        let Some(dir) = dirs.iter().find(|d| d.join("colors.toml").is_file()) else {
            return false;
        };
        let paths = [
            dir.join("colors.toml"),
            dir.parent().unwrap_or(dir).join("theme.name"),
            dir.join("solitaire/back.svg"),
            dir.join("solitaire/back.png"),
        ];
        let signature = paths
            .iter()
            .filter_map(|p| {
                fs::metadata(p).ok().map(|m| {
                    (
                        p.clone(),
                        m.ino(),
                        m.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                        m.len(),
                    )
                })
            })
            .collect::<Vec<_>>();
        if signature == self.signature {
            return false;
        }
        let Ok(bytes) = read_bounded(&paths[0], 65536) else {
            return false;
        };
        let Ok(raw) = String::from_utf8(bytes) else {
            return false;
        };
        let Ok(raw) = raw.parse::<toml::Table>() else {
            return false;
        };
        let colors = ["background", "foreground", "accent"]
            .map(|k| raw.get(k).and_then(|v| v.as_str()).and_then(hex));
        let [Some(bg), Some(fg), Some(ac)] = colors else {
            return false;
        };
        self.palette = Palette::new(bg, fg, ac);
        self.name = read_bounded(&paths[1], 1024)
            .ok()
            .and_then(|b| String::from_utf8(b).ok())
            .map(|s| s.trim().replace('-', " ").chars().take(80).collect())
            .unwrap_or_else(|| "Omarchy theme".into());
        self.error.clear();
        self.art = None;
        if let Some(p) = paths[2..].iter().find(|p| p.is_file()) {
            match validate_art(p) {
                Ok(art) => self.art = Some(art),
                Err(e) => self.error = e,
            }
        }
        self.signature = signature;
        true
    }
    pub fn invalidate(&mut self) {
        self.signature.clear();
    }
}
