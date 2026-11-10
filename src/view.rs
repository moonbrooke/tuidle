use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::game::{LetterState, MAX_GUESSES, WORD_LEN};
use crate::model::{Model, StatusKind};
use crate::styles::{
    footer_style, info_style, lose_style, subtitle_style, tile_style, title_style, warn_style,
    win_style, TILE_WIDTH,
};

pub fn board_width() -> usize {
    WORD_LEN * TILE_WIDTH + (WORD_LEN - 1)
}

pub fn content_width() -> usize {
    10 * TILE_WIDTH + 9
}

fn tile_span(letter: Option<char>, state: LetterState) -> Span<'static> {
    let ch = letter.filter(|c| c.is_ascii_uppercase()).unwrap_or(' ');
    let text = match state {
        LetterState::Correct
        | LetterState::Present
        | LetterState::Absent
        | LetterState::Current => format!(" {ch} "),
        _ => format!("[{ch}]"),
    };
    Span::styled(text, tile_style(state))
}

fn center_line(line: Line<'static>, width: usize, extra_left: usize) -> Line<'static> {
    let line_width = line.width();
    // Match Lipgloss: centre the block first, then apply the left margin.
    let pad = width.saturating_sub(line_width) / 2 + extra_left;
    if pad == 0 {
        return line;
    }

    let mut spans = Vec::with_capacity(line.spans.len() + 1);
    spans.push(Span::raw(" ".repeat(pad)));
    spans.extend(line.spans);
    Line::from(spans)
}

fn status_text(model: &Model) -> String {
    if !model.status.text.is_empty() {
        return model.status.text.clone();
    }
    if model.game_over() {
        "Press R or Enter to play again".to_string()
    } else {
        "Type a guess, then press Enter".to_string()
    }
}

fn status_style(kind: StatusKind) -> Style {
    match kind {
        StatusKind::Win => win_style(),
        StatusKind::Lose => lose_style(),
        StatusKind::Warn => warn_style(),
        _ => info_style(),
    }
}

fn help_line(model: &Model) -> Line<'static> {
    let bindings: &[(&str, &str)] = if model.game_over() {
        &[("[r / enter]", "restart,"), ("[ctrl+c / esc]", "quit")]
    } else {
        &[
            ("[a-z]", "type,"),
            ("[enter]", "submit,"),
            ("[backspace]", "delete,"),
            ("[ctrl+c / esc]", "quit"),
        ]
    };

    let mut spans = Vec::with_capacity(bindings.len() * 2);
    for (i, (key, desc)) in bindings.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", footer_style()));
        }
        spans.push(Span::styled(format!("{key} {desc}"), footer_style()));
    }
    Line::from(spans)
}

pub fn content_lines(model: &Model, width: u16) -> Vec<Line<'static>> {
    let layout_width = (width as usize).max(content_width());
    let shake = model.shake_offset();
    let mut lines = Vec::with_capacity(14);

    lines.push(center_line(
        Line::from(Span::styled("TUIDLE", title_style())),
        layout_width,
        0,
    ));
    lines.push(center_line(
        Line::from(Span::styled(
            "Guess the 5-letter word in 6 tries",
            subtitle_style(),
        )),
        layout_width,
        0,
    ));
    lines.push(Line::default());

    for row in 0..MAX_GUESSES {
        let mut spans = Vec::with_capacity(WORD_LEN * 2);
        for col in 0..WORD_LEN {
            if col > 0 {
                spans.push(Span::raw(" "));
            }
            let (letter, state) = model.cell(row, col);
            spans.push(tile_span(letter, state));
        }
        lines.push(center_line(Line::from(spans), layout_width, shake));
    }

    lines.push(Line::default());

    for row in ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"] {
        let mut spans = Vec::with_capacity(row.len() * 2);
        for (i, ch) in row.bytes().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            let idx = (ch - b'A') as usize;
            spans.push(tile_span(Some(ch as char), model.keyboard[idx]));
        }
        lines.push(center_line(Line::from(spans), layout_width, shake));
    }

    lines.push(Line::default());
    lines.push(center_line(
        Line::from(Span::styled(
            status_text(model),
            status_style(model.status.kind),
        )),
        layout_width,
        0,
    ));
    lines.push(center_line(help_line(model), layout_width, 0));

    if !model.dict.source.is_empty() {
        lines.push(center_line(
            Line::from(Span::styled(
                format!(
                    "Word list: {} ({} words)",
                    model.dict.source,
                    model.dict.all.len()
                ),
                footer_style(),
            )),
            layout_width,
            0,
        ));
    }

    lines
}

/// Plain-text rendering used by tests. Styles are intentionally dropped.
pub fn view_text(model: &Model, width: u16) -> String {
    content_lines(model, width)
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render(frame: &mut Frame, model: &Model) {
    let area = frame.area();
    let lines = content_lines(model, area.width);
    let content_height = lines.len() as u16;
    let y = if area.height > content_height {
        (area.height - content_height) / 2
    } else {
        0
    };

    let rect = Rect::new(0, y, area.width, area.height.saturating_sub(y));
    frame.render_widget(Paragraph::new(lines), rect);
}
