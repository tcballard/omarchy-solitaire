//! Klondike rules and the original Python save representation.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card(pub u8, pub bool);
impl Card {
    pub fn rank(self) -> u8 {
        self.0 % 13 + 1
    }
    pub fn suit(self) -> u8 {
        self.0 / 13
    }
    pub fn red(self) -> bool {
        matches!(self.suit(), 1 | 2)
    }
    pub fn label(self) -> &'static str {
        [
            "", "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
        ][self.rank() as usize]
    }
    pub fn name(self) -> String {
        format!(
            "{} of {}",
            self.label(),
            ["clubs", "diamonds", "hearts", "spades"][self.suit() as usize]
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub seed: u64,
    pub draw: usize,
    pub moves: u64,
    pub score: u64,
    pub passes: u64,
    pub piles: [Vec<Card>; 13],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub version: u32,
    pub state: State,
    pub history: Vec<State>,
}
pub type Move = (usize, usize, usize);
impl Game {
    pub fn new(seed: u64, draw: usize) -> Self {
        assert!(matches!(draw, 1 | 3));
        let mut deck: Vec<_> = (0..52).map(|n| Card(n, false)).collect();
        let mut rng = PythonRandom::new(seed);
        for i in (1..deck.len()).rev() {
            let j = rng.below(i + 1);
            deck.swap(i, j);
        }
        let mut piles: [Vec<Card>; 13] = Default::default();
        for (column, pile) in piles.iter_mut().skip(6).enumerate() {
            for _ in 0..=column {
                pile.push(deck.pop().unwrap());
            }
            pile.last_mut().unwrap().1 = true;
        }
        piles[0] = deck;
        Self {
            version: 1,
            state: State {
                seed,
                draw,
                moves: 0,
                score: 0,
                passes: 0,
                piles,
            },
            history: Vec::new(),
        }
    }
    pub fn random(draw: usize) -> Self {
        Self::new(rand::random::<u64>() & i64::MAX as u64, draw)
    }
    pub fn won(&self) -> bool {
        self.home() == 52
    }
    pub fn home(&self) -> usize {
        self.state.piles[2..6].iter().map(Vec::len).sum()
    }
    pub fn can_complete(&self) -> bool {
        !self.won()
            && self.state.piles[0].is_empty()
            && self.state.piles[1].is_empty()
            && self.state.piles[6..].iter().flatten().all(|c| c.1)
    }
    pub fn draw(&mut self) -> bool {
        if self.won() || (self.state.piles[0].is_empty() && self.state.piles[1].is_empty()) {
            return false;
        }
        self.history.push(self.state.clone());
        if self.state.piles[0].is_empty() {
            self.state.piles[0] = std::mem::take(&mut self.state.piles[1])
                .into_iter()
                .rev()
                .map(|c| Card(c.0, false))
                .collect();
            self.state.passes += 1;
            self.state.score = self.state.score.saturating_sub(20);
        } else {
            for _ in 0..self.state.draw {
                if let Some(c) = self.state.piles[0].pop() {
                    self.state.piles[1].push(Card(c.0, true));
                }
            }
        }
        self.state.moves += 1;
        true
    }
    pub fn undo(&mut self) -> bool {
        if let Some(state) = self.history.pop() {
            self.state = state;
            true
        } else {
            false
        }
    }
    pub fn movable(&self, source: usize, index: usize) -> bool {
        if !(1..13).contains(&source) {
            return false;
        }
        let pile = &self.state.piles[source];
        if index >= pile.len() {
            return false;
        }
        let tail = &pile[index..];
        if source < 6 {
            return tail.len() == 1 && tail[0].1;
        }
        tail.iter().all(|c| c.1)
            && tail
                .windows(2)
                .all(|w| w[0].rank() == w[1].rank() + 1 && w[0].red() != w[1].red())
    }
    pub fn legal(&self, source: usize, index: usize, target: usize) -> bool {
        if self.won()
            || source == target
            || !(2..13).contains(&target)
            || !self.movable(source, index)
        {
            return false;
        }
        let tail = &self.state.piles[source][index..];
        let card = tail[0];
        let top = self.state.piles[target].last();
        if target < 6 {
            tail.len() == 1
                && top.map_or(card.rank() == 1, |c| {
                    c.suit() == card.suit() && c.rank() + 1 == card.rank()
                })
        } else {
            top.map_or(card.rank() == 13, |c| {
                c.1 && c.rank() == card.rank() + 1 && c.red() != card.red()
            })
        }
    }
    pub fn play(&mut self, s: usize, i: usize, t: usize) -> bool {
        if !self.legal(s, i, t) {
            return false;
        }
        self.history.push(self.state.clone());
        let tail = self.state.piles[s].split_off(i);
        self.state.piles[t].extend(tail);
        if t < 6 {
            self.state.score += 10;
        } else if (2..6).contains(&s) {
            self.state.score = self.state.score.saturating_sub(15);
        } else if s == 1 {
            self.state.score += 5;
        }
        if s >= 6 {
            if let Some(c) = self.state.piles[s].last_mut() {
                if !c.1 {
                    c.1 = true;
                    self.state.score += 5;
                }
            }
        }
        self.state.moves += 1;
        true
    }
    pub fn legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        for s in 1..13 {
            for i in 0..self.state.piles[s].len() {
                for t in 2..13 {
                    if self.legal(s, i, t) {
                        moves.push((s, i, t));
                    }
                }
            }
        }
        moves
    }
    pub fn hint(&self) -> Option<Move> {
        let mut best = None;
        let mut weight = -1;
        for (s, i, t) in self.legal_moves() {
            if (2..6).contains(&s) || (t >= 6 && self.state.piles[t].is_empty() && s >= 6 && i == 0)
            {
                continue;
            }
            let reveal = s >= 6 && i > 0 && !self.state.piles[s][i - 1].1;
            let score = if reveal { 100 } else { 0 }
                + if t < 6 { 40 } else { 0 }
                + if s == 1 { 20 } else { 0 };
            if score > weight {
                best = Some((s, i, t));
                weight = score;
            }
        }
        best.or_else(|| {
            if !self.state.piles[0].is_empty() || !self.state.piles[1].is_empty() {
                Some((0, 0, 1))
            } else {
                None
            }
        })
    }
    pub fn completion_move(&self) -> Option<Move> {
        if !self.can_complete() {
            return None;
        }
        for s in 6..13 {
            if let Some(i) = self.state.piles[s].len().checked_sub(1) {
                for t in 2..6 {
                    if self.legal(s, i, t) {
                        return Some((s, i, t));
                    }
                }
            }
        }
        None
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported saved game version".into());
        }
        self.state.validate()?;
        for state in &self.history {
            state.validate()?;
        }
        Ok(())
    }
}
impl State {
    pub fn validate(&self) -> Result<(), String> {
        let bad = || Err("Invalid saved card arrangement".to_string());
        if !matches!(self.draw, 1 | 3)
            || [self.seed, self.moves, self.score, self.passes]
                .iter()
                .any(|v| *v > i64::MAX as u64)
        {
            return bad();
        }
        let mut ids: Vec<_> = self.piles.iter().flatten().map(|c| c.0).collect();
        ids.sort_unstable();
        if ids != (0..52).collect::<Vec<_>>() {
            return bad();
        }
        if self.piles[0].iter().any(|c| c.1) || self.piles[1..6].iter().flatten().any(|c| !c.1) {
            return bad();
        }
        for pile in &self.piles[2..6] {
            for (i, c) in pile.iter().enumerate() {
                if c.suit() != pile[0].suit() || c.rank() as usize != i + 1 {
                    return bad();
                }
            }
        }
        for pile in &self.piles[6..] {
            if pile.last().is_some_and(|c| !c.1) {
                return bad();
            }
            for w in pile.windows(2) {
                if w[0].1 && (!w[1].1 || w[0].rank() != w[1].rank() + 1 || w[0].red() == w[1].red())
                {
                    return bad();
                }
            }
        }
        Ok(())
    }
}

// CPython's integer-seeded MT19937 and getrandbits rejection shuffle.
// Keeping this stable makes Restart reproduce deals made by the Python app.
struct PythonRandom {
    mt: [u32; 624],
    index: usize,
}
impl PythonRandom {
    fn new(seed: u64) -> Self {
        let mut r = Self {
            mt: [0; 624],
            index: 624,
        };
        r.mt[0] = 19650218;
        for i in 1..624 {
            r.mt[i] = 1812433253u32
                .wrapping_mul(r.mt[i - 1] ^ (r.mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        let keys = if seed >> 32 == 0 {
            vec![seed as u32]
        } else {
            vec![seed as u32, (seed >> 32) as u32]
        };
        let (mut i, mut j) = (1, 0);
        for _ in 0..624 {
            r.mt[i] = (r.mt[i] ^ (r.mt[i - 1] ^ (r.mt[i - 1] >> 30)).wrapping_mul(1664525))
                .wrapping_add(keys[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= 624 {
                r.mt[0] = r.mt[623];
                i = 1;
            }
            if j >= keys.len() {
                j = 0;
            }
        }
        for _ in 0..623 {
            r.mt[i] = (r.mt[i] ^ (r.mt[i - 1] ^ (r.mt[i - 1] >> 30)).wrapping_mul(1566083941))
                .wrapping_sub(i as u32);
            i += 1;
            if i >= 624 {
                r.mt[0] = r.mt[623];
                i = 1;
            }
        }
        r.mt[0] = 0x80000000;
        r
    }
    fn next(&mut self) -> u32 {
        if self.index >= 624 {
            for i in 0..624 {
                let y = (self.mt[i] & 0x80000000) | (self.mt[(i + 1) % 624] & 0x7fffffff);
                self.mt[i] =
                    self.mt[(i + 397) % 624] ^ (y >> 1) ^ if y & 1 != 0 { 0x9908b0df } else { 0 };
            }
            self.index = 0;
        }
        let mut y = self.mt[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;
        y
    }
    fn below(&mut self, n: usize) -> usize {
        let bits = usize::BITS - n.leading_zeros();
        loop {
            let k = (self.next() >> (32 - bits)) as usize;
            if k < n {
                return k;
            }
        }
    }
}
