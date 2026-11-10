use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph};
use ratatui::Frame;

use crate::game::{LetterState, MAX_GUESSES, WORD_LEN};
use crate::model::{Model, StatusKind};
use crate::styles::{
    divider_style, footer_style, help_style, info_style, key_hint_style, key_style, lose_style,
    panel_border_style, subtitle_style, tile_style, title_style, warn_style, win_style, ABSENT_BG,
    CORRECT_BG, PANEL_BG, PRESENT_BG, TILE_WIDTH,
};
use crate::VERSION;

const H_PADDING: usize = 3;
const V_PADDING: usize = 1;
const KEYBOARD_ROWS: [&str; 3] = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];
const KEYBOARD_WIDTH: usize = 10 * TILE_WIDTH + 9;

pub fn board_width() -> usize {
    WORD_LEN * TILE_WIDTH + (WORD_LEN - 1)
}

/// Content width shared by the header, board, keyboard, and footer.
pub fn content_width() -> usize {
    KEYBOARD_WIDTH + 4
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

fn title_line() -> Line<'static> {
    Line::from(vec![
        Span::styled("■ ", Style::default().fg(CORRECT_BG)),
        Span::styled("■ ", Style::default().fg(PRESENT_BG)),
        Span::styled("■  ", Style::default().fg(ABSENT_BG)),
        Span::styled("TUIDLE", title_style()),
    ])
}

fn divider_line() -> Line<'static> {
    Line::from(Span::styled("──────────  ·  ──────────", divider_style()))
}

fn board_tile(letter: Option<char>, state: LetterState) -> Span<'static> {
    let ch = letter
        .filter(|c| c.is_ascii_uppercase())
        .unwrap_or(match state {
            LetterState::Future | LetterState::ActiveEmpty => '·',
            _ => ' ',
        });
    Span::styled(format!(" {ch} "), tile_style(state))
}

fn key_span(letter: char, state: LetterState) -> Span<'static> {
    Span::styled(format!(" {letter} "), key_style(state))
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

fn status_icon(kind: StatusKind) -> &'static str {
    match kind {
        StatusKind::Win => "✓",
        StatusKind::Lose => "✕",
        StatusKind::Warn => "!",
        _ => "›",
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

fn status_line(model: &Model) -> Line<'static> {
    let kind = model.status.kind;
    let text = status_text(model);
    let label = format!(" {} {} ", status_icon(kind), text);
    Line::from(Span::styled(label, status_style(kind)))
}

fn help_line(model: &Model) -> Line<'static> {
    let bindings: &[(&str, &str)] = if model.game_over() {
        &[("r / enter", "restart"), ("esc", "quit")]
    } else {
        &[
            ("a-z", "type"),
            ("enter", "submit"),
            ("bksp", "del"),
            ("esc", "quit"),
        ]
    };

    let mut spans = Vec::with_capacity(bindings.len() * 3);
    for (i, (key, desc)) in bindings.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", help_style()));
        }
        spans.push(Span::styled(*key, key_hint_style()));
        spans.push(Span::styled(format!(" {desc}"), help_style()));
    }
    Line::from(spans)
}

#[allow(clippy::manual_is_multiple_of)] // Keep the crate's Rust 1.80 MSRV.
fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

pub fn content_lines(model: &Model, width: u16) -> Vec<Line<'static>> {
    let layout_width = width as usize;
    let shake = model.shake_offset();
    let mut lines = Vec::with_capacity(17);

    lines.push(center_line(title_line(), layout_width, 0));
    lines.push(center_line(
        Line::from(Span::styled(
            "Guess the 5-letter word in 6 tries",
            subtitle_style(),
        )),
        layout_width,
        0,
    ));
    lines.push(center_line(divider_line(), layout_width, 0));
    lines.push(Line::default());

    // Board: only this area shakes, like the original game.
    for row in 0..MAX_GUESSES {
        let mut spans = Vec::with_capacity(WORD_LEN * 2);
        for col in 0..WORD_LEN {
            if col > 0 {
                spans.push(Span::raw(" "));
            }
            let (letter, state) = model.cell(row, col);
            spans.push(board_tile(letter, state));
        }
        lines.push(center_line(Line::from(spans), layout_width, shake));
    }

    lines.push(Line::default());

    // Each keyboard row is centred independently so the short rows do not
    // sag to the left.
    for row in KEYBOARD_ROWS {
        let mut spans = Vec::with_capacity(row.len() * 2);
        for (i, ch) in row.bytes().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            let idx = (ch - b'A') as usize;
            spans.push(key_span(ch as char, model.keyboard[idx]));
        }
        lines.push(center_line(Line::from(spans), layout_width, 0));
    }

    lines.push(Line::default());
    lines.push(center_line(status_line(model), layout_width, 0));
    lines.push(center_line(help_line(model), layout_width, 0));

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

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width, height)
}

fn panel_block(model: &Model, v_padding: usize) -> Block<'static> {
    let words = format!(" {} words ", thousands(model.dict.all.len()));
    let version = format!(" v{VERSION} ");

    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(panel_border_style())
        .padding(Padding::new(
            H_PADDING as u16,
            H_PADDING as u16,
            v_padding as u16,
            v_padding as u16,
        ))
        .style(Style::default().bg(PANEL_BG))
        .title_bottom(
            Line::from(Span::styled(words, footer_style()))
                .left_aligned()
                .style(Style::default().bg(PANEL_BG)),
        )
        .title_bottom(
            Line::from(Span::styled(version, footer_style()))
                .right_aligned()
                .style(Style::default().bg(PANEL_BG)),
        )
}

pub fn render(frame: &mut Frame, model: &Model) {
    let area = frame.area();
    let card_width = content_width() + 2 * H_PADDING + 2;
    let lines = content_lines(model, content_width() as u16);
    let full_height = lines.len() + 2 * V_PADDING + 2;
    let compact_height = lines.len() + 2;

    let (v_padding, card_height) = if area.height as usize >= full_height {
        (V_PADDING, full_height)
    } else if area.height as usize >= compact_height {
        (0, compact_height)
    } else {
        (V_PADDING, full_height)
    };

    if area.width as usize >= card_width && area.height as usize >= card_height {
        let rect = centered_rect(area, card_width as u16, card_height as u16);
        let paragraph = Paragraph::new(lines)
            .block(panel_block(model, v_padding))
            .style(Style::default().bg(PANEL_BG));
        frame.render_widget(paragraph, rect);
        return;
    }

    // Small terminals fall back to the bare, unboxed layout.
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
