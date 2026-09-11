use omarchy_solitaire::{
    game::{Card, Game},
    storage::{self, Session, SessionLock},
    theme::{self, Theme},
};
use std::fs;
#[test]
fn python_save_preserves_preferences_stats_time_and_backup() {
    let d = tempfile::tempdir().unwrap();
    let bytes = include_bytes!("fixtures/python-session.json");
    fs::write(d.path().join("session.json"), bytes).unwrap();
    let loaded = storage::load(d.path());
    assert!(loaded.writable);
    let s = loaded.session;
    assert_eq!(s.elapsed, 123);
    assert_eq!(s.session, "python-migration-fixture");
    assert!(s.preferences.reduced_motion);
    assert!(!s.preferences.follow_theme);
    assert_eq!(s.stats["1"].won, 1);
    assert_eq!(s.theme_name, "Saved theme");
    assert_eq!(s.game.state, Game::new(42, 3).state);
    assert_eq!(
        fs::read(d.path().join("session-before-rust.json")).unwrap(),
        bytes
    );
    storage::save(d.path(), &s).unwrap();
    assert_eq!(storage::load(d.path()).session.game, s.game);
    assert_eq!(
        fs::read(d.path().join("session-before-rust.json")).unwrap(),
        bytes
    );
}
#[test]
fn atomic_saves_are_private_and_recover_corruption() {
    use std::os::unix::fs::PermissionsExt;
    let d = tempfile::tempdir().unwrap();
    storage::save(d.path(), &Session::default()).unwrap();
    assert_eq!(
        fs::metadata(d.path().join("session.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    fs::write(d.path().join("session.json"), b"{broken").unwrap();
    let l = storage::load(d.path());
    assert!(l.writable);
    assert!(l.message.contains("recovery"));
    let backup = fs::read_dir(d.path())
        .unwrap()
        .filter_map(Result::ok)
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("session-unreadable-")
        })
        .unwrap();
    assert_eq!(fs::read(backup.path()).unwrap(), b"{broken");
}
#[test]
fn bounded_reads_and_failed_recovery_do_not_overwrite_original() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("large"), [1; 20]).unwrap();
    assert!(storage::read_bounded(&d.path().join("large"), 10).is_err());
    fs::create_dir(d.path().join("session.json")).unwrap();
    let l = storage::load(d.path());
    assert!(!l.writable);
    assert!(d.path().join("session.json").is_dir());
}
#[test]
fn locks_exclude_rust_and_legacy_instances() {
    let d = tempfile::tempdir().unwrap();
    let l = SessionLock::acquire(d.path()).unwrap();
    assert!(SessionLock::acquire(d.path()).is_err());
    assert!(d.path().join("session.lock").exists());
    drop(l);
    assert!(!d.path().join("session.lock").exists());
    fs::write(d.path().join("session.lock"), b"legacy").unwrap();
    assert!(SessionLock::acquire(d.path()).is_err());
    assert_eq!(fs::read(d.path().join("session.lock")).unwrap(), b"legacy");
}
#[test]
fn stats_count_each_deal_and_win_only_once() {
    let mut s = Session::new(1);
    s.game.draw();
    s.record_move();
    s.game.undo();
    s.record_move();
    assert_eq!(s.stats["1"].played, 1);
    s.deal(1, true);
    s.game.draw();
    s.record_move();
    assert_eq!(s.stats["1"].played, 1);
    s.game.state.piles = Default::default();
    for suit in 0..4 {
        s.game.state.piles[suit + 2] = (0..13).map(|r| Card((suit * 13 + r) as u8, true)).collect();
    }
    s.elapsed = 100;
    s.record_move();
    s.record_move();
    assert_eq!(s.stats["1"].won, 1);
    assert_eq!(s.stats["1"].best, 100);
    s.deal(1, false);
    s.game.draw();
    s.record_move();
    assert_eq!(s.stats["1"].played, 2);
}
#[test]
fn save_cannot_reference_external_artwork() {
    let mut v: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/python-session.json")).unwrap();
    v["preferences"]["customArt"] = "../../elsewhere.svg".into();
    assert!(Session::parse(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn theme_replacement_retains_last_good_palette() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join("theme");
    fs::create_dir(&dir).unwrap();
    let p = dir.join("colors.toml");
    fs::write(
        &p,
        "background='#112233'\nforeground='#eeeeee'\naccent='#99ccff'",
    )
    .unwrap();
    let mut t = Theme::default();
    assert!(t.refresh_at(std::slice::from_ref(&dir)));
    let old = t.palette.table;
    fs::write(&p, "not toml").unwrap();
    assert!(!t.refresh_at(std::slice::from_ref(&dir)));
    assert_eq!(t.palette.table, old);
    fs::rename(&dir, d.path().join("previous")).unwrap();
    fs::create_dir(&dir).unwrap();
    fs::write(
        &p,
        "background='#eeeeee'\nforeground='#222222'\naccent='#333333'",
    )
    .unwrap();
    assert!(t.refresh_at(&[dir]));
    assert_ne!(t.palette.table, old);
}
#[test]
fn theme_text_meets_contrast_in_light_dark_and_mid_grey() {
    for bg in ["#182622", "#eeeeee", "#777777", "#888888"] {
        let p = theme::Palette::new(
            theme::hex(bg).unwrap(),
            theme::hex("#888888").unwrap(),
            theme::hex("#999999").unwrap(),
        );
        for fg in [p.text, p.accent] {
            assert!(theme::contrast(fg, p.table) >= 4.5);
            assert!(theme::contrast(fg, p.surface) >= 4.5);
        }
    }
}
#[test]
fn svg_import_rejects_active_external_and_oversized_content() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("back.svg");
    for svg in [
        "<svg><script/></svg>",
        "<svg><image href='file:///etc/passwd'/></svg>",
        "<!DOCTYPE svg><svg/>",
        "<svg><path fill='url(https://example.com/x)'/></svg>",
    ] {
        fs::write(&p, svg).unwrap();
        assert!(theme::validate_art(&p).is_err());
    }
    fs::write(&p, include_bytes!("../assets/omarchy-logo.svg")).unwrap();
    assert!(theme::validate_art(&p).is_ok());
    fs::write(&p, vec![b' '; 2_000_001]).unwrap();
    assert!(theme::validate_art(&p).is_err());
}
#[test]
fn png_import_checks_dimensions_and_decodes() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("back.png");
    image::RgbaImage::new(10, 14).save(&p).unwrap();
    assert!(theme::validate_art(&p).is_ok());
    image::RgbaImage::new(4097, 1).save(&p).unwrap();
    assert!(theme::validate_art(&p).is_err());
}
#[test]
fn stale_local_lock_is_recovered_but_foreign_lock_is_retained() {
    let d = tempfile::tempdir().unwrap();
    let host = fs::read_to_string("/etc/hostname").unwrap();
    fs::write(
        d.path().join("session.lock"),
        format!("2147483647\nomarchy-solitaire\n{}\n", host.trim()),
    )
    .unwrap();
    let lock = SessionLock::acquire(d.path()).unwrap();
    drop(lock);
    fs::write(
        d.path().join("session.lock"),
        "2147483647\nomarchy-solitaire\nother-machine\n",
    )
    .unwrap();
    assert!(SessionLock::acquire(d.path()).is_err());
    assert!(d.path().join("session.lock").exists());
}
