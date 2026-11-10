use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rand::rngs::StdRng;

use crate::game::{
    evaluate, plural, update_keyboard, win_message, LetterState, MAX_GUESSES, WORD_LEN,
};
use crate::words::{pick_target, Dictionary};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatusKind {
    #[default]
    None,
    Info,
    Warn,
    Win,
    Lose,
}

#[derive(Clone, Debug, Default)]
pub struct StatusMessage {
    pub text: String,
    pub kind: StatusKind,
    pub id: u64,
    pub sticky: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyOutcome {
    Handled,
    Quit,
}

pub struct Model {
    pub dict: Dictionary,
    pub rng: StdRng,
    pub forced_target: String,

    pub target: String,
    pub guesses: Vec<String>,
    pub feedback: [[LetterState; WORD_LEN]; MAX_GUESSES],

    pub current: String,
    pub keyboard: [LetterState; 26],

    pub won: bool,
    pub lost: bool,

    pub status: StatusMessage,
    pub status_seq: u64,

    pub shake_id: u64,
    pub shake_left: u8,

    pub width: u16,
    pub height: u16,

    status_clear_at: Option<Instant>,
    shake_next_at: Option<Instant>,
}

impl Model {
    pub fn new(dict: Dictionary, rng: StdRng, forced_target: &str) -> Self {
        let mut model = Self {
            dict,
            rng,
            forced_target: forced_target.to_uppercase(),
            target: String::new(),
            guesses: Vec::new(),
            feedback: [[LetterState::Unused; WORD_LEN]; MAX_GUESSES],
            current: String::new(),
            keyboard: [LetterState::Unused; 26],
            won: false,
            lost: false,
            status: StatusMessage::default(),
            status_seq: 0,
            shake_id: 0,
            shake_left: 0,
            width: 0,
            height: 0,
            status_clear_at: None,
            shake_next_at: None,
        };
        model.reset();
        model
    }

    pub fn reset(&mut self) {
        self.target = pick_target(&self.dict, &mut self.rng, &self.forced_target);
        self.guesses.clear();
        self.feedback = [[LetterState::Unused; WORD_LEN]; MAX_GUESSES];
        self.current.clear();
        self.keyboard = [LetterState::Unused; 26];
        self.won = false;
        self.lost = false;
        self.status = StatusMessage::default();
        self.shake_left = 0;
        self.shake_id += 1;
        self.status_clear_at = None;
        self.shake_next_at = None;
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    pub fn set_transient(&mut self, text: &str, kind: StatusKind, shake: bool) {
        self.set_transient_at(text, kind, shake, Instant::now());
    }

    pub(crate) fn set_transient_at(
        &mut self,
        text: &str,
        kind: StatusKind,
        shake: bool,
        now: Instant,
    ) {
        self.status_seq += 1;
        let id = self.status_seq;
        self.status = StatusMessage {
            text: text.to_string(),
            kind,
            id,
            sticky: false,
        };
        self.status_clear_at = Some(now + Duration::from_millis(1500));

        if shake {
            self.shake_id += 1;
            self.shake_left = 6;
            self.shake_next_at = Some(now + Duration::from_millis(50));
        }
    }

    pub fn set_sticky(&mut self, text: &str, kind: StatusKind) {
        self.status_seq += 1;
        self.status = StatusMessage {
            text: text.to_string(),
            kind,
            id: self.status_seq,
            sticky: true,
        };
        self.status_clear_at = None;
    }

    pub fn has_guessed(&self, word: &str) -> bool {
        self.guesses.iter().any(|guess| guess == word)
    }

    pub fn submit(&mut self) {
        let guess = self.current.clone();

        if guess.len() < WORD_LEN {
            self.set_transient("Not enough letters", StatusKind::Warn, true);
            return;
        }
        if !self.dict.valid.contains(&guess) {
            self.set_transient("Not in word list", StatusKind::Warn, true);
            return;
        }
        if self.has_guessed(&guess) {
            self.set_transient("Already guessed", StatusKind::Warn, true);
            return;
        }

        let row = self.guesses.len();
        let feedback = evaluate(&self.target, &guess);
        self.feedback[row] = feedback;
        update_keyboard(&mut self.keyboard, &guess, &feedback);
        self.guesses.push(guess.clone());
        self.current.clear();
        if !self.status.sticky {
            self.status = StatusMessage::default();
        }

        if guess == self.target {
            self.won = true;
            let n = self.guesses.len();
            let message = format!(
                "{} You won in {} {}!",
                win_message(n),
                n,
                plural(n, "guess", "guesses")
            );
            self.set_sticky(&message, StatusKind::Win);
        } else if self.guesses.len() >= MAX_GUESSES {
            self.lost = true;
            let message = format!("Game over! The word was {}", self.target);
            self.set_sticky(&message, StatusKind::Lose);
        }
    }

    pub fn game_over(&self) -> bool {
        self.won || self.lost
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> KeyOutcome {
        if key.kind == KeyEventKind::Release {
            return KeyOutcome::Handled;
        }

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        match key.code {
            KeyCode::Char('c') if ctrl => return KeyOutcome::Quit,
            KeyCode::Esc => return KeyOutcome::Quit,
            _ => {}
        }

        if self.game_over() {
            let restart = match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') if !ctrl && !alt => true,
                KeyCode::Enter if !ctrl && !alt => true,
                _ => false,
            };
            if restart {
                self.reset();
            }
            return KeyOutcome::Handled;
        }

        match key.code {
            KeyCode::Enter if !ctrl && !alt => self.submit(),
            KeyCode::Backspace => {
                self.current.pop();
            }
            KeyCode::Char('h') if ctrl => {
                self.current.pop();
            }
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Tab => {}
            KeyCode::Char(' ') => {}
            KeyCode::Char(c)
                if !ctrl && !alt && c.is_ascii_alphabetic() && self.current.len() < WORD_LEN =>
            {
                self.current.push(c.to_ascii_uppercase());
                if !self.status.sticky {
                    self.status = StatusMessage::default();
                }
            }
            _ => {}
        }

        KeyOutcome::Handled
    }

    /// Time until the next animation/status deadline, capped so the event loop
    /// wakes up regularly even when idle.
    pub fn next_timeout(&self, now: Instant) -> Duration {
        let mut timeout = Duration::from_millis(250);
        if let Some(at) = self.status_clear_at {
            timeout = timeout.min(at.saturating_duration_since(now));
        }
        if let Some(at) = self.shake_next_at {
            timeout = timeout.min(at.saturating_duration_since(now));
        }
        timeout
    }

    pub fn tick(&mut self, now: Instant) {
        self.tick_at(now);
    }

    pub(crate) fn tick_at(&mut self, now: Instant) {
        if let Some(at) = self.status_clear_at {
            if now >= at {
                self.status_clear_at = None;
                if !self.status.sticky {
                    self.status = StatusMessage::default();
                }
            }
        }

        if let Some(at) = self.shake_next_at {
            if now >= at {
                if self.shake_left > 0 {
                    self.shake_left -= 1;
                    if self.shake_left > 0 {
                        self.shake_next_at = Some(now + Duration::from_millis(50));
                    } else {
                        self.shake_next_at = None;
                    }
                } else {
                    self.shake_next_at = None;
                }
            }
        }
    }

    pub fn shake_offset(&self) -> usize {
        const OFFSETS: [usize; 7] = [0, 2, 0, 1, 0, 2, 0];
        let left = self.shake_left as usize;
        if left > 0 && left < OFFSETS.len() {
            OFFSETS[left]
        } else {
            0
        }
    }

    pub fn cell(&self, row: usize, col: usize) -> (Option<char>, LetterState) {
        if row < self.guesses.len() {
            let guess = &self.guesses[row];
            let letter = guess.as_bytes().get(col).map(|b| *b as char);
            (letter, self.feedback[row][col])
        } else if row == self.guesses.len() && !self.game_over() {
            if col < self.current.len() {
                let letter = self.current.as_bytes().get(col).map(|b| *b as char);
                (letter, LetterState::Current)
            } else {
                (None, LetterState::ActiveEmpty)
            }
        } else {
            (None, LetterState::Future)
        }
    }
}
