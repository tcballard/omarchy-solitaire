use eframe::egui::{
    self, pos2, vec2, Context, Event, Key, Modifiers, PointerButton, RawInput, Rect,
};
use omarchy_solitaire::{
    app::{Action, Dialog, SolitaireApp},
    game::Game,
    storage,
};
struct Harness {
    ctx: Context,
    app: SolitaireApp,
    _dir: tempfile::TempDir,
    time: f64,
    size: egui::Vec2,
}
impl Harness {
    fn new(w: f32, h: f32) -> Self {
        let ctx = Context::default();
        let dir = tempfile::tempdir().unwrap();
        let mut app = SolitaireApp::new(&ctx, dir.path().into(), None);
        app.session.game = Game::new(42, 1);
        app.session.preferences.reduced_motion = true;
        let mut h = Self {
            ctx,
            app,
            _dir: dir,
            time: 0.,
            size: vec2(w, h),
        };
        h.frame(vec![]);
        h.frame(vec![]);
        h
    }
    fn frame(&mut self, events: Vec<Event>) {
        self.time += 0.1;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0., 0.), self.size)),
            time: Some(self.time),
            events,
            focused: true,
            ..Default::default()
        };
        let out = self.ctx.run(input, |ctx| self.app.ui(ctx));
        assert!(!out.shapes.is_empty());
    }
    fn click(&mut self, x: f32, y: f32) {
        let pos = pos2(x, y);
        self.frame(vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]);
        self.frame(vec![Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }]);
    }
    fn key(&mut self, key: Key, modifiers: Modifiers) {
        self.frame(vec![Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers,
        }]);
        self.frame(vec![Event::Key {
            key,
            physical_key: Some(key),
            pressed: false,
            repeat: false,
            modifiers,
        }]);
    }
}
#[test]
fn stock_click_and_keyboard_undo_persist() {
    let mut h = Harness::new(1120., 800.);
    h.click(60., 140.);
    assert_eq!(h.app.session.game.state.moves, 1);
    assert_eq!(
        storage::load(h._dir.path()).session.game,
        h.app.session.game
    );
    h.key(Key::Z, Modifiers::CTRL);
    assert_eq!(h.app.session.game.state.moves, 0);
    h.key(Key::Space, Modifiers::NONE);
    assert_eq!(h.app.session.game.state.moves, 1);
}
#[test]
fn compact_window_and_modal_shortcuts_preserve_table() {
    let mut h = Harness::new(800., 600.);
    h.click(60., 140.);
    assert_eq!(h.app.session.game.state.moves, 1);
    h.key(Key::N, Modifiers::CTRL);
    assert_eq!(h.app.dialog, Some(Dialog::New));
    h.key(Key::Z, Modifiers::CTRL);
    h.key(Key::Space, Modifiers::NONE);
    assert_eq!(h.app.session.game.state.moves, 1);
    h.key(Key::Escape, Modifiers::NONE);
    assert!(h.app.dialog.is_none());
    h.key(Key::Space, Modifiers::NONE);
    assert_eq!(h.app.session.game.state.moves, 2);
    h.app.dialog = Some(Dialog::Deck);
    h.frame(vec![]);
    h.app.dialog = Some(Dialog::Help);
    h.frame(vec![]);
}
#[test]
fn selection_hint_and_illegal_moves_are_safe() {
    let mut h = Harness::new(1120., 800.);
    h.app.act(Action::Hint);
    assert!(h.app.hint_target.is_some());
    let before = h.app.session.game.clone();
    h.app.act(Action::Move(0, 0, 2));
    assert_eq!(h.app.session.game, before);
    h.app.act(Action::Clear);
    assert!(h.app.selection.is_none());
    assert!(h.app.hint_target.is_none());
}
#[test]
fn pointer_drag_executes_a_legal_tableau_move() {
    let mut h = Harness::new(1120., 800.);
    // Find a deterministic initial top-card move, then drive the same pointer path a user takes.
    let (s, i, t) = loop {
        if let Some(m) = h
            .app
            .session
            .game
            .legal_moves()
            .into_iter()
            .find(|&(s, i, t)| s >= 6 && t >= 6 && i + 1 == h.app.session.game.state.piles[s].len())
        {
            break m;
        }
        h.app.session.game = Game::new(h.app.session.game.state.seed + 1, 1);
    };
    h.frame(vec![]);
    // 16px margins, 124px cards, 36.67px gutters, tableau starts at y=313.
    let w = 124.;
    let gap = (1088. - 7. * w) / 6.;
    let from = pos2(
        16. + (s - 6) as f32 * (w + gap) + w / 2.,
        313. + i as f32 * w * 0.12 + 30.,
    );
    let to = pos2(
        16. + (t - 6) as f32 * (w + gap) + w / 2.,
        313. + h.app.session.game.state.piles[t].len().saturating_sub(1) as f32 * w * 0.12 + 35.,
    );
    h.frame(vec![
        Event::PointerMoved(from),
        Event::PointerButton {
            pos: from,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
    ]);
    h.frame(vec![Event::PointerMoved(from + vec2(12., 0.))]);
    h.frame(vec![Event::PointerMoved(to)]);
    h.frame(vec![Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    }]);
    assert_eq!(
        h.app.session.game.state.moves, 1,
        "drag from {from:?} to {to:?}, move {s},{i},{t}: {}",
        h.app.message
    );
    assert_eq!(h.app.session.game.history.len(), 1);
}
#[test]
fn timers_pause_in_dialogs_and_do_not_count_suspend_gap() {
    let mut h = Harness::new(1120., 800.);
    h.app.act(Action::Draw);
    for _ in 0..12 {
        h.frame(vec![]);
    }
    assert!(h.app.session.elapsed >= 1);
    h.app.dialog = Some(Dialog::Help);
    let before = h.app.session.elapsed;
    for _ in 0..20 {
        h.frame(vec![]);
    }
    assert_eq!(h.app.session.elapsed, before);
    h.app.dialog = None;
    h.time += 600.;
    h.frame(vec![]);
    assert_eq!(h.app.session.elapsed, before);
}
#[test]
fn board_accepts_space_immediately_after_launch() {
    let mut h = Harness::new(800., 600.);
    h.key(Key::Space, Modifiers::NONE);
    assert_eq!(h.app.session.game.state.moves, 1);
}
#[test]
fn quit_shortcut_requests_close_without_reentering_input_lock() {
    let mut h = Harness::new(800., 600.);
    let out = h.ctx.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0., 0.), h.size)),
            time: Some(h.time + 0.1),
            events: vec![Event::Key {
                key: Key::Q,
                physical_key: Some(Key::Q),
                pressed: true,
                repeat: false,
                modifiers: Modifiers::CTRL,
            }],
            ..Default::default()
        },
        |ctx| h.app.ui(ctx),
    );
    assert!(out.viewport_output.values().any(|v| v
        .commands
        .iter()
        .any(|c| matches!(c, egui::ViewportCommand::Close))));
}
