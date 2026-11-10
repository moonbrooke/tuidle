use ratatui::style::{Color, Modifier, Style};

use crate::game::LetterState;

// Board colours.
pub const CORRECT_BG: Color = Color::Rgb(0x4f, 0xa3, 0x6a);
pub const PRESENT_BG: Color = Color::Rgb(0xd4, 0xaa, 0x3f);
pub const ABSENT_BG: Color = Color::Rgb(0x3a, 0x3d, 0x45);

pub const BRIGHT_FG: Color = Color::Rgb(0xf7, 0xf8, 0xf4);
pub const DARK_FG: Color = Color::Rgb(0x15, 0x17, 0x1b);
pub const MUTED_FG: Color = Color::Rgb(0x8b, 0x92, 0xa1);
pub const DIM_FG: Color = Color::Rgb(0x56, 0x5b, 0x66);

// Board and keyboard surfaces.
pub const FUTURE_BG: Color = Color::Rgb(0x1a, 0x1c, 0x22);
pub const CURRENT_BG: Color = Color::Rgb(0x2c, 0x30, 0x39);
pub const KEY_BG: Color = Color::Rgb(0x31, 0x35, 0x3e);
pub const KEY_FG: Color = Color::Rgb(0xe2, 0xe5, 0xea);
pub const KEY_DIM_FG: Color = Color::Rgb(0x5f, 0x65, 0x70);

// Panel chrome.
pub const PANEL_BG: Color = Color::Rgb(0x11, 0x13, 0x18);
pub const BORDER_FG: Color = Color::Rgb(0x35, 0x3a, 0x45);
pub const TITLE_FG: Color = Color::Rgb(0xff, 0xff, 0xff);
pub const SUBTITLE_FG: Color = Color::Rgb(0x8b, 0x92, 0xa1);
pub const ACCENT_FG: Color = Color::Rgb(0x8c, 0xd1, 0x9e);
pub const FOOTER_FG: Color = Color::Rgb(0x6b, 0x72, 0x80);

// Status pills.
pub const INFO_BG: Color = Color::Rgb(0x22, 0x25, 0x2c);
pub const WARN_BG: Color = Color::Rgb(0x6b, 0x35, 0x3a);
pub const LOSE_BG: Color = Color::Rgb(0x7a, 0x35, 0x3f);

pub const TILE_WIDTH: usize = 3;

pub fn title_style() -> Style {
    Style::default().fg(TITLE_FG).add_modifier(Modifier::BOLD)
}

pub fn subtitle_style() -> Style {
    Style::default().fg(SUBTITLE_FG)
}

pub fn divider_style() -> Style {
    Style::default().fg(BORDER_FG)
}

pub fn panel_border_style() -> Style {
    Style::default().fg(BORDER_FG)
}

pub fn footer_style() -> Style {
    Style::default().fg(FOOTER_FG)
}

pub fn help_style() -> Style {
    Style::default().fg(MUTED_FG)
}

pub fn key_hint_style() -> Style {
    Style::default().fg(ACCENT_FG).add_modifier(Modifier::BOLD)
}

pub fn win_style() -> Style {
    Style::default()
        .fg(BRIGHT_FG)
        .bg(CORRECT_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn lose_style() -> Style {
    Style::default()
        .fg(BRIGHT_FG)
        .bg(LOSE_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn warn_style() -> Style {
    Style::default()
        .fg(BRIGHT_FG)
        .bg(WARN_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn info_style() -> Style {
    Style::default().fg(MUTED_FG).bg(INFO_BG)
}

/// Style for a board tile.
pub fn tile_style(state: LetterState) -> Style {
    let base = Style::default().add_modifier(Modifier::BOLD);
    match state {
        LetterState::Correct => base.fg(BRIGHT_FG).bg(CORRECT_BG),
        LetterState::Present => base.fg(DARK_FG).bg(PRESENT_BG),
        LetterState::Absent => base.fg(BRIGHT_FG).bg(ABSENT_BG),
        LetterState::Current => base.fg(BRIGHT_FG).bg(CURRENT_BG),
        LetterState::Future => base.fg(DIM_FG).bg(FUTURE_BG),
        LetterState::Unused => base.fg(KEY_FG).bg(KEY_BG),
        LetterState::ActiveEmpty => base.fg(DIM_FG).bg(CURRENT_BG),
    }
}

/// Style for a keyboard keycap.
pub fn key_style(state: LetterState) -> Style {
    let base = Style::default().add_modifier(Modifier::BOLD);
    match state {
        LetterState::Correct => base.fg(BRIGHT_FG).bg(CORRECT_BG),
        LetterState::Present => base.fg(DARK_FG).bg(PRESENT_BG),
        LetterState::Absent => base.fg(KEY_DIM_FG).bg(KEY_BG),
        _ => base.fg(KEY_FG).bg(KEY_BG),
    }
}
