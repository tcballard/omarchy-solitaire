use crate::game::Game;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const SAVE_LIMIT: usize = 32_000_000;
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Record {
    pub played: u64,
    pub won: u64,
    pub best: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Preferences {
    pub pattern: usize,
    pub finish: String,
    pub reduced_motion: bool,
    pub follow_theme: bool,
    pub custom_art: String,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            pattern: 0,
            finish: "matte".into(),
            reduced_motion: false,
            follow_theme: true,
            custom_art: String::new(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub game: Game,
    #[serde(default)]
    pub elapsed: u64,
    pub session: String,
    #[serde(default)]
    pub counted: bool,
    #[serde(default)]
    pub started: bool,
    #[serde(default)]
    pub stats: BTreeMap<String, Record>,
    #[serde(default)]
    pub preferences: Preferences,
    #[serde(default, rename = "themeSnapshot")]
    pub theme_snapshot: BTreeMap<String, String>,
    #[serde(default, rename = "themeName")]
    pub theme_name: String,
    #[serde(default, rename = "lockedArt")]
    pub locked_art: String,
}
impl Default for Session {
    fn default() -> Self {
        Self::new(1)
    }
}
impl Session {
    pub fn new(draw: usize) -> Self {
        Self {
            game: Game::random(draw),
            elapsed: 0,
            session: stamp(),
            counted: false,
            started: false,
            stats: BTreeMap::new(),
            preferences: Preferences::default(),
            theme_snapshot: BTreeMap::new(),
            theme_name: String::new(),
            locked_art: String::new(),
        }
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let s: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        s.game.validate()?;
        if s.elapsed >= 1 << 31
            || s.session.len() > 80
            || s.preferences.pattern > 3
            || !["matte", "holographic"].contains(&s.preferences.finish.as_str())
        {
            return Err("Invalid saved preferences or time".into());
        }
        if !["", "custom-back.svg", "custom-back.png"].contains(&s.preferences.custom_art.as_str())
            || !["", "locked-theme.svg", "locked-theme.png"].contains(&s.locked_art.as_str())
        {
            return Err("Invalid saved artwork name".into());
        }
        if s.stats
            .values()
            .any(|r| [r.played, r.won, r.best].iter().any(|n| *n >= 1 << 31))
        {
            return Err("Invalid saved statistics".into());
        }
        Ok(s)
    }
    pub fn record_move(&mut self) {
        let record = self
            .stats
            .entry(self.game.state.draw.to_string())
            .or_default();
        if !self.started {
            record.played += 1;
            self.started = true;
        }
        if self.game.won() && !self.counted {
            record.won += 1;
            record.best = if record.best == 0 {
                self.elapsed
            } else {
                record.best.min(self.elapsed)
            };
            self.counted = true;
        }
    }
    pub fn deal(&mut self, draw: usize, restart: bool) {
        self.game = if restart {
            Game::new(self.game.state.seed, self.game.state.draw)
        } else {
            Game::random(draw)
        };
        self.elapsed = 0;
        if !restart {
            self.session = stamp();
            self.started = false;
            self.counted = false;
        }
    }
}
pub fn xdg(variable: &str, fallback: &str) -> PathBuf {
    env::var_os(variable)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(fallback))
}
pub fn state_dir() -> PathBuf {
    xdg("XDG_STATE_HOME", ".local/state").join("omarchy-solitaire")
}
pub fn stamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string()
}
pub fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err(format!("File exceeds {limit} bytes"));
    }
    Ok(bytes)
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut f = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    f.write_all(bytes)
        .and_then(|_| f.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    f.persist(path).map_err(|e| e.to_string())?;
    File::open(dir)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn save(dir: &Path, s: &Session) -> Result<(), String> {
    let bytes = serde_json::to_vec(s).map_err(|e| e.to_string())?;
    if bytes.len() > SAVE_LIMIT {
        return Err(
            "The full undo history exceeds the save limit; this game could not be saved.".into(),
        );
    }
    atomic_write(&dir.join("session.json"), &bytes)
}
pub struct Loaded {
    pub session: Session,
    pub message: String,
    pub writable: bool,
}
pub fn load(dir: &Path) -> Loaded {
    let path = dir.join("session.json");
    if !path.exists() {
        return Loaded {
            session: Session::default(),
            message: "Draw a card to begin.".into(),
            writable: true,
        };
    }
    match read_bounded(&path, SAVE_LIMIT).and_then(|b| Session::parse(&b)) {
        Ok(session) => {
            // Retain a byte-for-byte pre-migration copy once, including unknown fields.
            let backup = dir.join("session-before-rust.json");
            if !backup.exists() {
                if let Err(e) =
                    read_bounded(&path, SAVE_LIMIT).and_then(|b| atomic_write(&backup, &b))
                {
                    return Loaded {
                        session,
                        message: format!(
                            "Couldn't preserve the original save: {e}. Saving is disabled."
                        ),
                        writable: false,
                    };
                }
            }
            Loaded {
                session,
                message: "Welcome back. Your table is just as you left it.".into(),
                writable: true,
            }
        }
        Err(_) => {
            // Stream the recovery copy even when the original exceeds the read limit.
            let copy = (|| -> Result<(), String> {
                let mut input = File::open(&path).map_err(|e| e.to_string())?;
                let mut output = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
                std::io::copy(&mut input, &mut output).map_err(|e| e.to_string())?;
                output.as_file().sync_all().map_err(|e| e.to_string())?;
                output
                    .persist(dir.join(format!("session-unreadable-{}.json", stamp())))
                    .map_err(|e| e.to_string())?;
                Ok(())
            })();
            Loaded {
                session: Session::default(),
                writable: copy.is_ok(),
                message: if copy.is_ok() {
                    "The save couldn't be read. A recovery copy was kept; a fresh deal is ready."
                        .into()
                } else {
                    "Couldn't preserve the unreadable save. Saving is disabled; the original is untouched.".into()
                },
            }
        }
    }
}
pub struct SessionLock {
    _file: File,
    qt_path: PathBuf,
}
impl SessionLock {
    pub fn acquire(dir: &Path) -> Result<Self, String> {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(dir.join("rust-session.lock"))
            .map_err(|e| e.to_string())?;
        file.try_lock_exclusive()
            .map_err(|_| "Another Solitaire instance is using this save directory.".to_string())?;
        // Reserve Qt's lock path too, so an older installed Python app cannot write concurrently.
        let qt_path = dir.join("session.lock");
        let host = fs::read_to_string("/etc/hostname").unwrap_or_default();
        // Recover a crashed local instance only when its PID is demonstrably absent.
        // Keep unknown hosts, malformed locks and inaccessible /proc entries untouched.
        if let Ok(bytes) = read_bounded(&qt_path, 4096) {
            if let Ok(text) = String::from_utf8(bytes) {
                let lines: Vec<_> = text.lines().collect();
                if lines.len() >= 3 && !host.trim().is_empty() && lines[2] == host.trim() {
                    if let Ok(pid) = lines[0].parse::<u32>() {
                        if pid > 0
                            && matches!(
                                PathBuf::from(format!("/proc/{pid}")).try_exists(),
                                Ok(false)
                            )
                        {
                            fs::remove_file(&qt_path).map_err(|e| e.to_string())?;
                        }
                    }
                }
            }
        }
        let mut qt=OpenOptions::new().write(true).create_new(true).open(&qt_path).map_err(|_|"The save directory is locked. Close the other Solitaire instance. After a crash, remove session.lock only after confirming no instance is running.".to_string())?;
        if let Err(e) = write!(
            qt,
            "{}\nomarchy-solitaire\n{}\n",
            std::process::id(),
            host.trim()
        )
        .and_then(|_| qt.sync_all())
        {
            let _ = fs::remove_file(&qt_path);
            return Err(e.to_string());
        }
        Ok(Self {
            _file: file,
            qt_path,
        })
    }
}
impl Drop for SessionLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.qt_path);
    }
}
