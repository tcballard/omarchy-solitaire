use omarchy_solitaire::game::{Card, Game, State};
use serde::Deserialize;
#[derive(Deserialize)]
struct Fixture {
    initial: State,
    actions: Vec<Vec<serde_json::Value>>,
    expected: Game,
}
#[test]
fn python_shuffle_moves_scores_and_undo_are_identical() {
    let fixtures: Vec<Fixture> =
        serde_json::from_str(include_str!("fixtures/python-games.json")).unwrap();
    for f in fixtures {
        let mut g = Game::new(f.initial.seed, f.initial.draw);
        assert_eq!(g.state, f.initial);
        for a in f.actions {
            match a[0].as_str().unwrap() {
                "draw" => {
                    g.draw();
                }
                "undo" => {
                    g.undo();
                }
                "move" => {
                    assert!(g.play(
                        a[1].as_u64().unwrap() as usize,
                        a[2].as_u64().unwrap() as usize,
                        a[3].as_u64().unwrap() as usize
                    ));
                }
                _ => panic!("Unknown oracle action"),
            }
            g.validate().unwrap();
        }
        assert_eq!(g, f.expected);
    }
}
#[test]
fn random_play_conserves_cards_and_undo_restores_every_state() {
    for seed in 0..40 {
        let mut g = Game::new(seed, if seed % 2 == 0 { 1 } else { 3 });
        let initial = g.state.clone();
        for step in 0..150 {
            let before = g.clone();
            let m = g.legal_moves();
            if !m.is_empty() && step % 4 != 0 {
                let (s, i, t) = m[(seed as usize + step) % m.len()];
                assert!(g.play(s, i, t));
            } else {
                g.draw();
            }
            g.validate().unwrap();
            let json = serde_json::to_vec(&g).unwrap();
            assert_eq!(g, serde_json::from_slice::<Game>(&json).unwrap());
            if g.state != before.state {
                let mut undone = g.clone();
                assert!(undone.undo());
                assert_eq!(undone, before);
            }
        }
        while g.undo() {}
        assert_eq!(g.state, initial);
    }
}
#[test]
fn undo_is_not_limited_to_500_moves() {
    let mut g = Game::new(7, 1);
    let initial = g.state.clone();
    for _ in 0..550 {
        assert!(g.draw());
    }
    for _ in 0..550 {
        assert!(g.undo());
    }
    assert_eq!(g.state, initial);
    assert!(!g.undo());
}
#[test]
fn invalid_input_and_moves_are_transactional() {
    let mut g = Game::new(0, 1);
    let initial = g.clone();
    for (s, i, t) in [
        (0, 0, 2),
        (99, 0, 3),
        (1, usize::MAX, 6),
        (6, 0, 99),
        (6, 0, 6),
    ] {
        assert!(!g.play(s, i, t));
        assert_eq!(g, initial);
    }
    g.state.piles[0][0] = g.state.piles[0][1];
    assert!(g.validate().is_err());
}
fn almost_won() -> Game {
    let mut g = Game::new(0, 1);
    g.state.piles = Default::default();
    for s in 0..4 {
        g.state.piles[s + 2] = (0..12).map(|r| Card((s * 13 + r) as u8, true)).collect();
        g.state.piles[s + 6].push(Card((s * 13 + 12) as u8, true));
    }
    g
}
#[test]
fn completion_is_conservative_and_each_step_is_undoable() {
    let mut g = almost_won();
    g.validate().unwrap();
    assert!(g.can_complete());
    let initial = g.state.clone();
    for _ in 0..4 {
        let (s, i, t) = g.completion_move().unwrap();
        assert!(g.play(s, i, t));
    }
    assert!(g.won());
    assert!(g.completion_move().is_none());
    assert!(!g.draw());
    for _ in 0..4 {
        assert!(g.undo());
    }
    assert_eq!(g.state, initial);
    let c = g.state.piles[6].pop().unwrap();
    g.state.piles[0].push(Card(c.0, false));
    assert!(!g.can_complete());
}
#[test]
fn only_kings_fill_empty_columns_and_foundations_require_suit_order() {
    let mut g = almost_won();
    assert!(g.legal(6, 0, 12));
    assert!(!g.legal(6, 0, 3));
    assert!(g.legal(6, 0, 2));
    assert!(g.play(6, 0, 2));
    assert!(!g.legal(2, 11, 6));
    assert!(g.legal(2, 12, 6));
}
#[test]
fn invalid_orientations_and_sequences_are_rejected() {
    let mut g = Game::new(1, 1);
    g.state.piles[0][0].1 = true;
    assert!(g.validate().is_err());
    let mut g = almost_won();
    g.state.piles[2].swap(0, 1);
    assert!(g.validate().is_err());
    let mut g = Game::new(1, 1);
    g.state.piles[12].last_mut().unwrap().1 = false;
    assert!(g.validate().is_err());
}
#[test]
fn draw_three_partial_packet_and_recycle_preserve_order() {
    let mut g = Game::new(42, 3);
    let stock = g.state.piles[0].clone();
    for _ in 0..8 {
        g.draw();
    }
    assert_eq!(g.state.piles[1].len(), 24);
    g.draw();
    assert_eq!(g.state.piles[0], stock);
    assert_eq!(g.state.passes, 1);
}
