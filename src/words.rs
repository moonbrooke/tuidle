use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::Path;

use rand::Rng;
use serde::Deserialize;

use crate::game::is_five_letters;

pub const EMBEDDED_ANSWERS: &str = include_str!("../tuidle-answers.txt");
pub const EMBEDDED_GUESSES: &str = include_str!("../tuidle-allowed-guesses.txt");

pub const ANSWERS_JSON_NAME: &str = "tuidle-answers.json";
pub const ANSWERS_TXT_NAME: &str = "tuidle-answers.txt";
pub const GUESSES_JSON_NAME: &str = "tuidle-allowed-guesses.json";
pub const GUESSES_TXT_NAME: &str = "tuidle-allowed-guesses.txt";

pub const FALLBACK_WORDS: &[&str] = &[
    "CRANE", "SLATE", "TRACE", "CRATE", "AUDIO", "RAISE", "ARISE", "STARE", "ABOUT", "ABOVE",
    "ACTOR", "AFTER", "AGAIN", "AGENT", "AGREE", "AHEAD", "ALARM", "ALBUM", "ALERT", "ALIVE",
    "ALLOW", "ALONE", "ALONG", "ALTER", "AMONG", "ANGER", "ANGLE", "ANGRY", "ANKLE", "APART",
    "APPLE", "APPLY", "ARENA", "ARGUE", "ARMOR", "ARRAY", "ARROW", "ASIDE", "ATLAS", "ATONE",
    "AWAKE", "AWARE", "AWFUL", "AZURE", "BACON", "BADGE", "BADLY", "BAKER", "BALLS", "BANDS",
    "BANJO", "BANKS", "BARGE", "BARKS", "BARON", "BASIC", "BASIN", "BATCH", "BATON", "BEACH",
    "BEARD", "BEAST", "BEGIN", "BEING", "BELLY", "BELOW", "BENCH", "BERRY", "BIRTH", "BLACK",
    "BLADE", "BLAME", "BLANK", "BLAST", "BLAZE", "BLEND", "BLESS", "BLIND", "BLOCK", "BLOOD",
];

#[derive(Clone, Debug, Default)]
pub struct Dictionary {
    pub valid: HashSet<String>,
    pub answers: Vec<String>,
    pub all: Vec<String>,
    pub source: String,
}

#[derive(Debug)]
pub enum WordsError {
    Empty,
    Io(std::io::Error),
    Json(serde_json::Error),
    Utf8(std::str::Utf8Error),
}

impl fmt::Display for WordsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WordsError::Empty => f.write_str("empty word list"),
            WordsError::Io(err) => write!(f, "{err}"),
            WordsError::Json(err) => write!(f, "{err}"),
            WordsError::Utf8(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for WordsError {}

impl From<std::io::Error> for WordsError {
    fn from(err: std::io::Error) -> Self {
        WordsError::Io(err)
    }
}

impl From<serde_json::Error> for WordsError {
    fn from(err: serde_json::Error) -> Self {
        WordsError::Json(err)
    }
}

impl From<std::str::Utf8Error> for WordsError {
    fn from(err: std::str::Utf8Error) -> Self {
        WordsError::Utf8(err)
    }
}

pub fn normalise_words<S: AsRef<str>>(raw: &[S]) -> Vec<String> {
    let mut seen = HashSet::with_capacity(raw.len());
    let mut out = Vec::with_capacity(raw.len());
    for word in raw {
        let word = word.as_ref().trim().to_uppercase();
        if !is_five_letters(&word) {
            continue;
        }
        if !seen.insert(word.clone()) {
            continue;
        }
        out.push(word);
    }
    out
}

pub fn build_dictionary<A: AsRef<str>, G: AsRef<str>>(
    answers_raw: &[A],
    guesses_raw: &[G],
    source: &str,
) -> Dictionary {
    let answers = normalise_words(answers_raw);
    let guesses = normalise_words(guesses_raw);

    let mut valid = HashSet::with_capacity(answers.len() + guesses.len());
    let mut all = Vec::with_capacity(answers.len() + guesses.len());
    for word in &answers {
        if valid.insert(word.clone()) {
            all.push(word.clone());
        }
    }
    for word in &guesses {
        if valid.insert(word.clone()) {
            all.push(word.clone());
        }
    }

    Dictionary {
        valid,
        answers,
        all,
        source: source.to_string(),
    }
}

#[derive(Deserialize)]
struct WordsObject {
    #[serde(default)]
    words: Vec<String>,
}

pub fn parse_words_json(data: &[u8]) -> Result<Vec<String>, WordsError> {
    let text = std::str::from_utf8(data)?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(WordsError::Empty);
    }
    if trimmed.starts_with('{') {
        let obj: WordsObject = serde_json::from_str(trimmed)?;
        Ok(obj.words)
    } else {
        let arr: Vec<String> = serde_json::from_str(trimmed)?;
        Ok(arr)
    }
}

pub fn parse_words_txt(data: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(data)
        .split(['\n', '\r', ',', ' ', '\t'])
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn load_word_file(path: &str) -> Result<Vec<String>, WordsError> {
    let data = fs::read(path)?;
    let text = String::from_utf8_lossy(&data);
    let trimmed = text.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return parse_words_json(&data);
    }
    let words = parse_words_txt(&data);
    if words.is_empty() {
        return Err(WordsError::Empty);
    }
    Ok(words)
}

/// Loads one word list, trying the explicit path, then the conventional
/// `.json` and `.txt` files, then the embedded list.
pub fn load_one_list(
    explicit: &str,
    json_name: &str,
    txt_name: &str,
    embedded: &str,
) -> (Vec<String>, String, bool) {
    if !explicit.is_empty() {
        if let Ok(words) = load_word_file(explicit) {
            if !normalise_words(&words).is_empty() {
                return (words, explicit.to_string(), true);
            }
        }
    }

    for name in [json_name, txt_name] {
        if !Path::new(name).exists() {
            continue;
        }
        if let Ok(words) = load_word_file(name) {
            if !normalise_words(&words).is_empty() {
                return (words, name.to_string(), true);
            }
        }
    }

    if !embedded.trim().is_empty() {
        let words = parse_words_txt(embedded.as_bytes());
        if !normalise_words(&words).is_empty() {
            return (words, format!("embedded {txt_name}"), true);
        }
    }

    (Vec::new(), String::new(), false)
}

pub fn load_dictionary(answers_path: &str, guesses_path: &str) -> Dictionary {
    let (mut answers_raw, mut answers_source, ok_answers) = load_one_list(
        answers_path,
        ANSWERS_JSON_NAME,
        ANSWERS_TXT_NAME,
        EMBEDDED_ANSWERS,
    );
    let (guesses_raw, guesses_source, _) = load_one_list(
        guesses_path,
        GUESSES_JSON_NAME,
        GUESSES_TXT_NAME,
        EMBEDDED_GUESSES,
    );

    if !ok_answers {
        answers_raw = FALLBACK_WORDS.iter().map(|w| (*w).to_string()).collect();
        answers_source = "built-in fallback".to_string();
    }

    let mut sources = Vec::with_capacity(2);
    if !answers_source.is_empty() {
        sources.push(answers_source);
    }
    if !guesses_source.is_empty() {
        sources.push(guesses_source);
    }

    build_dictionary(&answers_raw, &guesses_raw, &sources.join(" + "))
}

pub fn pick_target<R: Rng + ?Sized>(dict: &Dictionary, rng: &mut R, forced: &str) -> String {
    if !forced.is_empty() {
        return forced.to_uppercase();
    }

    let pool = if !dict.answers.is_empty() {
        &dict.answers
    } else if !dict.all.is_empty() {
        &dict.all
    } else {
        // This only happens for an empty hand-built dictionary.
        return FALLBACK_WORDS[rng.gen_range(0..FALLBACK_WORDS.len())].to_string();
    };

    pool[rng.gen_range(0..pool.len())].clone()
}

pub fn sorted_keys(valid: &HashSet<String>) -> Vec<String> {
    let mut keys: Vec<String> = valid.iter().cloned().collect();
    keys.sort();
    keys
}
