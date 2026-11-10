use std::env;
use std::fs;
use std::io;
use std::process::ExitCode;
use std::time::Instant;

use crossterm::event::{self, Event, KeyEventKind};
use rand::rngs::StdRng;
use rand::SeedableRng;

use tuidle::game::is_five_letters;
use tuidle::model::{KeyOutcome, Model};
use tuidle::view;
use tuidle::words::{
    load_dictionary, ANSWERS_JSON_NAME, ANSWERS_TXT_NAME, GUESSES_JSON_NAME, GUESSES_TXT_NAME,
};
use tuidle::VERSION;

#[derive(Default)]
struct Options {
    words: String,
    answers: String,
    guesses: String,
    target: String,
    version: bool,
    v: bool,
    help: bool,
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value {
        "1" | "t" | "T" | "true" | "TRUE" | "True" => Ok(true),
        "0" | "f" | "F" | "false" | "FALSE" | "False" => Ok(false),
        _ => Err(format!("invalid boolean value {value:?}")),
    }
}

fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Options, String> {
    let args: Vec<String> = args.into_iter().collect();
    let mut options = Options::default();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        // Go's flag package stops at "--" or the first non-flag argument.
        if arg == "--" || arg == "-" || !arg.starts_with('-') {
            break;
        }

        let stripped = arg.strip_prefix("--").unwrap_or(&arg[1..]);
        if stripped.is_empty() {
            break;
        }

        let (name, inline) = match stripped.split_once('=') {
            Some((name, value)) => (name, Some(value.to_string())),
            None => (stripped, None),
        };

        match name {
            "h" | "help" => {
                options.help = true;
                break;
            }
            "version" | "v" => {
                let enabled = match inline {
                    Some(value) => parse_bool(&value)?,
                    None => true,
                };
                if name == "version" {
                    options.version = enabled;
                } else {
                    options.v = enabled;
                }
            }
            "words" | "answers" | "guesses" | "target" => {
                let value = match inline {
                    Some(value) => value,
                    None => {
                        i += 1;
                        if i >= args.len() {
                            return Err(format!("flag needs an argument: -{name}"));
                        }
                        args[i].clone()
                    }
                };
                match name {
                    "words" => options.words = value,
                    "answers" => options.answers = value,
                    "guesses" => options.guesses = value,
                    "target" => options.target = value,
                    _ => unreachable!(),
                }
            }
            _ => return Err(format!("flag provided but not defined: -{name}")),
        }

        i += 1;
    }

    Ok(options)
}

fn usage_text() -> String {
    format!(
        "\
Usage of tuidle:
  -answers string
\tpath to answers/target list (.json or .txt); default: ./{ANSWERS_JSON_NAME}, then ./{ANSWERS_TXT_NAME}, then the embedded list
  -guesses string
\tpath to allowed-guesses list (.json or .txt); default: ./{GUESSES_JSON_NAME}, then ./{GUESSES_TXT_NAME}, then the embedded list
  -target string
\tforce the target word (5 letters, for testing); default: chosen at random from the answers list
  -v\tshorthand for -version
  -version
\tprint version and exit
  -words string
\tpath to word list (.json or .txt), used as both answers and guesses (legacy; prefer -answers / -guesses); default: ./{ANSWERS_JSON_NAME}, then ./{ANSWERS_TXT_NAME}, then the embedded list
"
    )
}

fn print_usage() {
    print!("{}", usage_text());
}

fn print_usage_err() {
    eprint!("{}", usage_text());
}

fn main() -> ExitCode {
    let options = match parse_args(env::args().skip(1)) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            print_usage_err();
            return ExitCode::from(2);
        }
    };

    if options.help {
        print_usage();
        return ExitCode::SUCCESS;
    }

    if options.version || options.v {
        println!("tuidle {VERSION}");
        return ExitCode::SUCCESS;
    }

    if !options.target.is_empty() && !is_five_letters(&options.target.to_uppercase()) {
        eprintln!("error: -target must be exactly 5 letters A-Z");
        return ExitCode::from(2);
    }

    for (name, path) in [
        ("-words", &options.words),
        ("-answers", &options.answers),
        ("-guesses", &options.guesses),
    ] {
        if !path.is_empty() {
            if let Err(err) = fs::metadata(path) {
                eprintln!(
                    "warning: {name} {path:?} not readable ({err}); falling back to the default lists"
                );
            }
        }
    }

    let mut answers_path = options.answers.clone();
    let mut guesses_path = options.guesses.clone();
    if !options.words.is_empty() {
        if answers_path.is_empty() {
            answers_path = options.words.clone();
        }
        if guesses_path.is_empty() {
            guesses_path = options.words.clone();
        }
    }

    let dict = load_dictionary(&answers_path, &guesses_path);
    if dict.all.is_empty() {
        eprintln!("error: no usable word list found");
        return ExitCode::from(1);
    }

    let rng = StdRng::from_entropy();
    let model = Model::new(dict, rng, &options.target);
    run_tui(model)
}

fn run_tui(mut model: Model) -> ExitCode {
    let mut terminal = match ratatui::try_init() {
        Ok(terminal) => terminal,
        Err(err) => {
            eprintln!("error: failed to start TUI: {err}");
            eprintln!("hint: run in a real terminal (Windows Terminal on Windows, any terminal on Linux); it cannot run in a pipe or redirected shell");
            return ExitCode::from(1);
        }
    };

    let result = run_loop(&mut terminal, &mut model);
    let restore_result = ratatui::try_restore();

    if let Err(err) = result {
        eprintln!("error: failed to start TUI: {err}");
        eprintln!("hint: run in a real terminal (Windows Terminal on Windows, any terminal on Linux); it cannot run in a pipe or redirected shell");
        return ExitCode::from(1);
    }

    if let Err(err) = restore_result {
        eprintln!("error: failed to restore terminal: {err}");
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}

fn run_loop(terminal: &mut ratatui::DefaultTerminal, model: &mut Model) -> io::Result<()> {
    let size = terminal.size()?;
    model.resize(size.width, size.height);

    loop {
        terminal.draw(|frame| view::render(frame, model))?;

        let timeout = model.next_timeout(Instant::now());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    if model.handle_key(key) == KeyOutcome::Quit {
                        break;
                    }
                }
                Event::Resize(width, height) => model.resize(width, height),
                _ => {}
            }
        }

        model.tick(Instant::now());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn parses_single_and_double_dash_flags() {
        let options = parse_args(args(&[
            "-answers",
            "a.json",
            "--guesses=g.txt",
            "-target",
            "crane",
            "-words=w.txt",
        ]))
        .expect("valid flags");

        assert_eq!(options.answers, "a.json");
        assert_eq!(options.guesses, "g.txt");
        assert_eq!(options.target, "crane");
        assert_eq!(options.words, "w.txt");
    }

    #[test]
    fn parses_version_aliases_and_booleans() {
        assert!(parse_args(args(&["-version"])).unwrap().version);
        assert!(parse_args(args(&["-v"])).unwrap().v);
        assert!(!parse_args(args(&["-v=false"])).unwrap().v);
        assert!(parse_args(args(&["--version=true"])).unwrap().version);
    }

    #[test]
    fn help_stops_parsing() {
        let options = parse_args(args(&["-h", "-bogus"])).expect("help wins");
        assert!(options.help);
    }

    #[test]
    fn positional_argument_stops_parsing() {
        let options = parse_args(args(&["positional", "-bogus"])).expect("stops at positional");
        assert!(!options.help);
        assert_eq!(options.answers, "");
    }

    #[test]
    fn rejects_unknown_flags_and_missing_values() {
        assert!(parse_args(args(&["-bogus"])).is_err());
        assert!(parse_args(args(&["-answers"])).is_err());
    }
}
