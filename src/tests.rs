use std::collections::HashSet;
use std::fs;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::rngs::StdRng;
use rand::SeedableRng;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use tempfile::tempdir;

use crate::game::*;
use crate::model::*;
use crate::view;
use crate::words::*;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn key_msg(s: &str) -> KeyEvent {
    match s {
        "enter" => key(KeyCode::Enter),
        "backspace" => key(KeyCode::Backspace),
        "ctrl+c" => ctrl('c'),
        "esc" => key(KeyCode::Esc),
        "up" => key(KeyCode::Up),
        "down" => key(KeyCode::Down),
        "left" => key(KeyCode::Left),
        "right" => key(KeyCode::Right),
        other => {
            let mut chars = other.chars();
            let ch = chars.next().expect("key string must not be empty");
            assert!(chars.next().is_none(), "key string must be one character");
            key(KeyCode::Char(ch))
        }
    }
}

#[test]
fn test_evaluate() {
    let cases: &[(&str, &str, [LetterState; WORD_LEN])] = &[
        (
            "CRANE",
            "CRANE",
            [
                LetterState::Correct,
                LetterState::Correct,
                LetterState::Correct,
                LetterState::Correct,
                LetterState::Correct,
            ],
        ),
        (
            "ROBOT",
            "SPEED",
            [
                LetterState::Absent,
                LetterState::Absent,
                LetterState::Absent,
                LetterState::Absent,
                LetterState::Absent,
            ],
        ),
        (
            "ROBOT",
            "BOOTS",
            [
                LetterState::Present,
                LetterState::Correct,
                LetterState::Present,
                LetterState::Present,
                LetterState::Absent,
            ],
        ),
        (
            "APPLE",
            "POPPY",
            [
                LetterState::Present,
                LetterState::Absent,
                LetterState::Correct,
                LetterState::Absent,
                LetterState::Absent,
            ],
        ),
        (
            "SPEED",
            "EERIE",
            [
                LetterState::Present,
                LetterState::Present,
                LetterState::Absent,
                LetterState::Absent,
                LetterState::Absent,
            ],
        ),
        (
            "CRANE",
            "SLATE",
            [
                LetterState::Absent,
                LetterState::Absent,
                LetterState::Correct,
                LetterState::Absent,
                LetterState::Correct,
            ],
        ),
    ];

    for (target, guess, want) in cases {
        assert_eq!(
            evaluate(target, guess),
            *want,
            "evaluate({target:?}, {guess:?})"
        );
    }
}

#[test]
fn test_update_keyboard_never_downgrades() {
    let mut kb = [LetterState::Unused; 26];
    let feedback = evaluate("SPEED", "EERIE");
    update_keyboard(&mut kb, "EERIE", &feedback);
    assert_eq!(kb[(b'E' - b'A') as usize], LetterState::Present);

    let absent = [LetterState::Absent; WORD_LEN];
    update_keyboard(&mut kb, "EXCEL", &absent);
    assert_eq!(kb[(b'E' - b'A') as usize], LetterState::Present);

    let correct = evaluate("EERIE", "EERIE");
    update_keyboard(&mut kb, "EERIE", &correct);
    assert_eq!(kb[(b'E' - b'A') as usize], LetterState::Correct);
}

#[test]
fn test_parse_words_json() {
    let objs = [
        r#"{"words":["crane","slate","crane","toolong","ab"]}"#,
        r#"["crane","slate"]"#,
    ];

    for s in objs {
        let words = parse_words_json(s.as_bytes()).expect("valid JSON");
        let got = normalise_words(&words);
        assert!(!got.is_empty());
        assert_eq!(got[0], "CRANE");
    }

    let words = parse_words_json(br#"{"words":["crane","slate","crane","toolong","ab"]}"#)
        .expect("valid JSON");
    let got = normalise_words(&words);
    assert_eq!(got, vec!["CRANE".to_string(), "SLATE".to_string()]);
}

#[test]
fn test_load_dictionary_from_temp_files() {
    let dir = tempdir().expect("temp dir");

    let json_path = dir.path().join("w.json");
    fs::write(&json_path, r#"{"words":["apple","robot","speed"]}"#).expect("write json");
    let words = load_word_file(json_path.to_str().expect("utf-8 path")).expect("load json");
    assert_eq!(words.len(), 3);

    let txt_path = dir.path().join("w.txt");
    fs::write(&txt_path, "crane\nslate\ntrace\n").expect("write txt");
    let raw = parse_words_txt(&fs::read(&txt_path).expect("read txt"));
    let dict = build_dictionary(&raw, &[] as &[String], "test.txt");

    assert_eq!(dict.valid.len(), 3);
    assert_eq!(dict.source, "test.txt");
    assert!(
        dict.valid.contains("CRANE"),
        "{:?}",
        sorted_keys(&dict.valid)
    );
}

#[test]
fn test_dictionary_answers_vs_guesses_split() {
    let answers = ["CRANE", "SLATE"];
    let guesses = ["AAHED", "ABACA"];
    let dict = build_dictionary(&answers, &guesses, "split");

    assert_eq!(dict.answers, vec!["CRANE".to_string(), "SLATE".to_string()]);

    for word in ["CRANE", "SLATE", "AAHED", "ABACA"] {
        assert!(
            dict.valid.contains(word),
            "{word} should be valid: {:?}",
            sorted_keys(&dict.valid)
        );
    }
    assert_eq!(dict.all.len(), 4);
    assert!(dict.valid.contains("CRANE"));

    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..200 {
        let got = pick_target(&dict, &mut rng, "");
        assert!(got == "CRANE" || got == "SLATE");
    }
}

#[test]
fn test_pick_target_uses_answers_only() {
    let answers = ["CRANE", "SLATE", "TRACE"];
    let guesses = ["AAHED", "ABACA", "ABACI"];
    let dict = build_dictionary(&answers, &guesses, "split");

    assert_eq!(
        pick_target(&dict, &mut StdRng::seed_from_u64(1), "robot"),
        "ROBOT"
    );

    let mut rng = StdRng::seed_from_u64(7);
    let mut seen = HashSet::new();
    for _ in 0..500 {
        let got = pick_target(&dict, &mut rng, "");
        assert!(
            answers.contains(&got.as_str()),
            "pickTarget returned {got:?}, not in answers {answers:?}"
        );
        seen.insert(got);
    }
    assert!(seen.len() >= 2, "pickTarget only ever returned {seen:?}");
}

#[test]
fn test_embedded_and_fallback_lists_are_usable() {
    let dict = build_dictionary(FALLBACK_WORDS, &[] as &[&str], "fallback");
    assert_eq!(dict.all.len(), FALLBACK_WORDS.len());
    assert_eq!(dict.answers.len(), FALLBACK_WORDS.len());

    let embedded = build_dictionary(
        &parse_words_txt(EMBEDDED_ANSWERS.as_bytes()),
        &parse_words_txt(EMBEDDED_GUESSES.as_bytes()),
        "embedded",
    );

    assert!(
        embedded.answers.len() >= 1000,
        "embedded answers parsed to only {} words",
        embedded.answers.len()
    );
    assert!(
        embedded.valid.len() >= 5000,
        "embedded union parsed to only {} words",
        embedded.valid.len()
    );
    assert!(embedded.valid.contains("CRANE"));
    assert!(embedded.valid.contains("AAHED"));
    assert!(embedded.answers.iter().any(|w| w == "CRANE"));
}

#[test]
fn test_load_dictionary_split_from_temp_files() {
    let dir = tempdir().expect("temp dir");
    let answers_path = dir.path().join("tuidle-answers.json");
    let guesses_path = dir.path().join("tuidle-allowed-guesses.txt");

    fs::write(&answers_path, r#"{"words":["crane","slate"]}"#).expect("write answers");
    fs::write(&guesses_path, "aahed\nabaca\n").expect("write guesses");

    let answers_str = answers_path.to_str().expect("utf-8 path");
    let guesses_str = guesses_path.to_str().expect("utf-8 path");

    let (answers_raw, answers_source, ok_answers) =
        load_one_list(answers_str, ANSWERS_JSON_NAME, ANSWERS_TXT_NAME, "");
    let (guesses_raw, guesses_source, ok_guesses) =
        load_one_list(guesses_str, GUESSES_JSON_NAME, GUESSES_TXT_NAME, "");

    assert!(ok_answers && ok_guesses);
    assert_eq!(answers_source, answers_str);
    assert_eq!(guesses_source, guesses_str);

    let source = format!("{answers_source} + {guesses_source}");
    let dict = build_dictionary(&answers_raw, &guesses_raw, &source);
    assert_eq!(dict.answers.len(), 2);

    for word in ["CRANE", "SLATE", "AAHED", "ABACA"] {
        assert!(dict.valid.contains(word), "{word} should be valid");
    }
}

fn test_model(target: &str) -> Model {
    let answers = [
        "CRANE", "SLATE", "ROBOT", "SPEED", "APPLE", "POPPY", "EERIE", "BOOTS", "TRACE", "CRATE",
    ];
    let dict = build_dictionary(&answers, &answers, "test");
    Model::new(dict, StdRng::seed_from_u64(1), target)
}

fn type_word(model: &mut Model, word: &str) {
    for ch in word.chars() {
        model.handle_key(key_msg(&ch.to_string()));
    }
    model.handle_key(key_msg("enter"));
}

#[test]
fn test_model_win_and_restart() {
    let mut model = test_model("CRANE");
    type_word(&mut model, "CRANE");

    assert!(model.won);
    assert_eq!(model.guesses.len(), 1);
    assert!(
        model.status.text.contains("won"),
        "win status = {:?}",
        model.status.text
    );

    model.handle_key(key_msg("r"));
    assert!(!model.won);
    assert!(model.guesses.is_empty());
    assert_eq!(model.target, "CRANE");

    type_word(&mut model, "CRANE");
    model.handle_key(key_msg("enter"));
    assert!(!model.won, "Enter after win should restart");
}

#[test]
fn test_model_rejects_invalid_word_and_short_guess() {
    let mut model = test_model("CRANE");
    type_word(&mut model, "ZZZZZ");
    assert!(model.guesses.is_empty());
    assert_eq!(model.status.text, "Not in word list");
    assert!(
        model.shake_left > 0,
        "expected board shake after invalid word"
    );

    let mut model = test_model("CRANE");
    for ch in "CRA".chars() {
        model.handle_key(key_msg(&ch.to_string()));
    }
    model.handle_key(key_msg("enter"));
    assert!(model.guesses.is_empty());
    assert_eq!(model.status.text, "Not enough letters");
}

#[test]
fn test_model_backspace_arrow_keys_and_letter_limit() {
    let mut model = test_model("CRANE");
    for ch in "CRANEX".chars() {
        model.handle_key(key_msg(&ch.to_string()));
    }
    assert_eq!(model.current, "CRANE", "5-letter limit");

    for k in ["up", "down", "left", "right"] {
        model.handle_key(key_msg(k));
    }
    assert_eq!(model.current, "CRANE", "arrow keys changed current");

    model.handle_key(key_msg("backspace"));
    assert_eq!(model.current, "CRAN");
}

#[test]
fn test_model_rejects_duplicate_guess() {
    let mut model = test_model("CRANE");
    type_word(&mut model, "SLATE");
    assert_eq!(model.guesses.len(), 1);

    type_word(&mut model, "SLATE");
    assert_eq!(model.guesses.len(), 1, "duplicate guess was submitted");
    assert_eq!(model.status.text, "Already guessed");
    assert!(model.shake_left > 0, "expected board shake after duplicate");
    assert!(!model.won && !model.lost);
}

#[test]
fn test_model_loses_after_six() {
    let mut model = test_model("CRANE");
    for word in ["SLATE", "TRACE", "CRATE", "ROBOT", "SPEED", "APPLE"] {
        type_word(&mut model, word);
    }

    assert!(model.lost);
    assert_eq!(model.guesses.len(), 6);
    assert!(
        model.status.text.contains("CRANE"),
        "loss status should reveal target: {:?}",
        model.status.text
    );
}

#[test]
fn test_model_quit() {
    let mut model = test_model("CRANE");
    for k in ["ctrl+c", "esc"] {
        assert_eq!(
            model.handle_key(key_msg(k)),
            KeyOutcome::Quit,
            "{k} did not quit"
        );
    }
}

#[test]
fn test_view_renders_board_keyboard_and_status() {
    let mut model = test_model("CRANE");
    model.current = "CRA".to_string();
    let rendered = view::view_text(&model, 80);

    for want in [
        "TUIDLE",
        "Guess the 5-letter word in 6 tries",
        "C",
        "R",
        "A",
        " Q ",
        " W ",
        " P ",
        " A ",
        " L ",
        " Z ",
        " M ",
        "Type a guess",
    ] {
        assert!(
            rendered.contains(want),
            "View() missing {want:?}:\n{rendered}"
        );
    }

    assert!(
        !rendered.contains('|'),
        "View() must not contain '|':\n{rendered}"
    );
    assert!(
        rendered.matches('·').count() >= 2,
        "expected empty slots in the active row:\n{rendered}"
    );
}

#[test]
fn test_ratatui_render_buffer() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    let mut model = test_model("CRANE");
    model.resize(80, 24);

    terminal
        .draw(|frame| view::render(frame, &model))
        .expect("draw view");

    let buffer = terminal.backend().buffer();
    let text = buffer
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    assert!(text.contains("╭"), "rendered buffer:\n{text}");
    assert!(text.contains("╰"), "rendered buffer:\n{text}");
    assert!(text.contains("TUIDLE"), "rendered buffer:\n{text}");
    assert!(text.contains(" Q "), "rendered buffer:\n{text}");
    assert!(text.contains("10 words"), "rendered buffer:\n{text}");
    assert!(
        text.contains("Guess the 5-letter word in 6 tries"),
        "rendered buffer:\n{text}"
    );
}

#[test]
fn test_ratatui_render_small_terminal_falls_back() {
    let backend = TestBackend::new(40, 14);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    let mut model = test_model("CRANE");
    model.resize(40, 14);

    terminal
        .draw(|frame| view::render(frame, &model))
        .expect("draw compact view");

    let buffer = terminal.backend().buffer();
    let text = buffer
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    assert!(text.contains("TUIDLE"), "rendered buffer:\n{text}");
    assert!(
        !text.contains('╭'),
        "small terminals should not draw the card border:\n{text}"
    );
}

#[test]
fn test_transient_status_clears_and_shake_ticks() {
    let mut model = test_model("CRANE");
    let now = Instant::now();

    model.set_transient_at("Not in word list", StatusKind::Warn, true, now);
    assert_eq!(model.status.text, "Not in word list");
    assert_eq!(model.shake_left, 6);
    assert_eq!(model.shake_offset(), 0);

    model.tick_at(now + Duration::from_millis(50));
    assert_eq!(model.shake_left, 5);
    assert_eq!(model.shake_offset(), 2);

    for step in 2..=6 {
        model.tick_at(now + Duration::from_millis(50 * step));
    }
    assert_eq!(model.shake_left, 0);
    assert_eq!(model.shake_offset(), 0);
    assert_eq!(model.status.text, "Not in word list");

    model.tick_at(now + Duration::from_millis(1500));
    assert_eq!(model.status.text, "");
}

#[test]
fn test_sticky_status_survives_timer() {
    let mut model = test_model("CRANE");
    let now = Instant::now();
    model.set_sticky("Game over! The word was CRANE", StatusKind::Lose);
    model.tick_at(now + Duration::from_secs(10));
    assert_eq!(model.status.text, "Game over! The word was CRANE");
    assert!(model.status.sticky);
}
