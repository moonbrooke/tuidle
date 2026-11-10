//! Pure game logic: scoring, keyboard precedence, and win messages.
//!
//! This module deliberately has no terminal or UI dependencies so it can be
//! tested in isolation, mirroring the original Go `game.go`.

pub const WORD_LEN: usize = 5;
pub const MAX_GUESSES: usize = 6;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LetterState {
    /// Not guessed yet.
    #[default]
    Unused,
    /// Not in target.
    Absent,
    /// In target, wrong position.
    Present,
    /// In target, right position.
    Correct,
    /// Rendering-only state for the current row.
    Current,
    /// Rendering-only state for an empty active cell.
    ActiveEmpty,
    /// Rendering-only state for a future row.
    Future,
}

/// Scores `guess` against `target` (both 5 uppercase ASCII letters).
///
/// Exact matches consume letter counts first, so repeated letters score
/// correctly.
pub fn evaluate(target: &str, guess: &str) -> [LetterState; WORD_LEN] {
    let mut result = [LetterState::Absent; WORD_LEN];
    if target.len() != WORD_LEN || guess.len() != WORD_LEN {
        return result;
    }

    let target = target.as_bytes();
    let guess = guess.as_bytes();

    let mut counts = [0i32; 26];
    for &c in target {
        if c.is_ascii_uppercase() {
            counts[(c - b'A') as usize] += 1;
        }
    }

    for i in 0..WORD_LEN {
        if guess[i] == target[i] {
            result[i] = LetterState::Correct;
            if target[i].is_ascii_uppercase() {
                counts[(target[i] - b'A') as usize] -= 1;
            }
        }
    }

    for i in 0..WORD_LEN {
        if result[i] == LetterState::Correct {
            continue;
        }
        let c = guess[i];
        if !c.is_ascii_uppercase() {
            result[i] = LetterState::Absent;
            continue;
        }
        let idx = (c - b'A') as usize;
        if counts[idx] > 0 {
            result[i] = LetterState::Present;
            counts[idx] -= 1;
        } else {
            result[i] = LetterState::Absent;
        }
    }

    result
}

/// Keeps better keyboard states from being overwritten.
pub fn letter_rank(state: LetterState) -> i32 {
    match state {
        LetterState::Correct => 3,
        LetterState::Present => 2,
        LetterState::Absent => 1,
        _ => 0,
    }
}

pub fn update_keyboard(
    keyboard: &mut [LetterState; 26],
    guess: &str,
    feedback: &[LetterState; WORD_LEN],
) {
    for (i, state) in feedback.iter().enumerate().take(WORD_LEN.min(guess.len())) {
        let c = guess.as_bytes()[i];
        if !c.is_ascii_uppercase() {
            continue;
        }
        let idx = (c - b'A') as usize;
        if letter_rank(*state) > letter_rank(keyboard[idx]) {
            keyboard[idx] = *state;
        }
    }
}

pub fn is_five_letters(s: &str) -> bool {
    s.len() == WORD_LEN && s.bytes().all(|b| b.is_ascii_uppercase())
}

pub fn win_message(n: usize) -> &'static str {
    match n {
        1 => "Genius!",
        2 => "Magnificent!",
        3 => "Impressive!",
        4 => "Splendid!",
        5 => "Great!",
        _ => "Phew!",
    }
}

pub fn plural(n: usize, one: &'static str, many: &'static str) -> &'static str {
    if n == 1 {
        one
    } else {
        many
    }
}
