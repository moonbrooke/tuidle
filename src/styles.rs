use ratatui::style::{Color, Modifier, Style};

use crate::game::LetterState;

pub const CORRECT_BG: Color = Color::Rgb(0x53, 0x8d, 0x4e);
pub const PRESENT_BG: Color = Color::Rgb(0xb5, 0x9f, 0x3b);
pub const ABSENT_BG: Color = Color::Rgb(0x78, 0x7c, 0x7e);
pub const BRIGHT_FG: Color = Color::Rgb(0xf8, 0xf8, 0xf2);
pub const DIM_FG: Color = Color::Rgb(0x56, 0x57, 0x58);
pub const ACTIVE_FG: Color = Color::Rgb(0xd7, 0xda, 0xdc);
pub const CURRENT_BG: Color = Color::Rgb(0x25, 0x25, 0x26);
pub const TITLE_FG: Color = Color::Rgb(0xff, 0xff, 0xff);
pub const SUBTITLE_FG: Color = Color::Rgb(0x9a, 0x9a, 0x9a);
pub const WARN_FG: Color = Color::Rgb(0xf2, 0xb8, 0xb5);
pub const WIN_FG: Color = Color::Rgb(0xa6, 0xd8, 0xa8);
pub const LOSE_FG: Color = Color::Rgb(0xf2, 0xb8, 0xb5);
pub const FOOTER_FG: Color = Color::Rgb(0x8a, 0x8a, 0x8a);

pub const TILE_WIDTH: usize = 3;

pub fn title_style() -> Style {
    Style::default().fg(TITLE_FG).add_modifier(Modifier::BOLD)
}

pub fn subtitle_style() -> Style {
    Style::default().fg(SUBTITLE_FG)
}

pub fn win_style() -> Style {
    Style::default().fg(WIN_FG).add_modifier(Modifier::BOLD)
}

pub fn lose_style() -> Style {
    Style::default().fg(LOSE_FG).add_modifier(Modifier::BOLD)
}

pub fn warn_style() -> Style {
    Style::default().fg(WARN_FG)
}

pub fn info_style() -> Style {
    Style::default().fg(SUBTITLE_FG)
}

pub fn footer_style() -> Style {
    Style::default().fg(FOOTER_FG)
}

pub fn tile_style(state: LetterState) -> Style {
    let base = Style::default().add_modifier(Modifier::BOLD);
    match state {
        LetterState::Correct => base.fg(BRIGHT_FG).bg(CORRECT_BG),
        LetterState::Present => base.fg(BRIGHT_FG).bg(PRESENT_BG),
        LetterState::Absent => base.fg(BRIGHT_FG).bg(ABSENT_BG),
        LetterState::Current => base.fg(BRIGHT_FG).bg(CURRENT_BG),
        LetterState::Future => base.fg(DIM_FG),
        LetterState::Unused | LetterState::ActiveEmpty => base.fg(ACTIVE_FG),
    }
}
