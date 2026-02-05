use ratatui::{
    prelude::*,
    widgets::Widget,
};

use crate::terminal::TerminalPane;

pub struct TerminalWidget<'a> {
    terminal: &'a TerminalPane,
}

impl<'a> TerminalWidget<'a> {
    pub fn new(terminal: &'a TerminalPane) -> Self {
        Self { terminal }
    }
}

impl<'a> Widget for TerminalWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let output = self.terminal.get_output();
        let lines: Vec<&str> = output.lines().collect();

        let visible_height = area.height as usize;
        let scroll_offset = self.terminal.scroll_offset();

        // Calculate which lines to show (from bottom)
        let total_lines = lines.len();
        let start = if total_lines > visible_height + scroll_offset {
            total_lines - visible_height - scroll_offset
        } else {
            0
        };
        let end = (start + visible_height).min(total_lines);

        for (i, line_idx) in (start..end).enumerate() {
            let y = area.y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            let line = lines.get(line_idx).unwrap_or(&"");

            // Parse ANSI escape codes and render with colors
            render_ansi_line(buf, area.x, y, area.width, line);
        }
    }
}

/// Simple ANSI escape code parser for terminal output
fn render_ansi_line(buf: &mut Buffer, x: u16, y: u16, width: u16, line: &str) {
    let mut current_x = x;
    let mut current_style = Style::default();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if current_x >= x + width {
            break;
        }

        if ch == '\x1b' {
            // Parse ANSI escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                let mut params = String::new();

                // Read parameters until we hit a letter
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == ';' {
                        params.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                // Read the command character
                if let Some(cmd) = chars.next() {
                    if cmd == 'm' {
                        // SGR (Select Graphic Rendition)
                        current_style = parse_sgr(&params, current_style);
                    }
                }
            }
        } else if ch == '\r' {
            // Carriage return - reset to start of line
            current_x = x;
        } else if ch == '\n' {
            // Newline - handled by line splitting
        } else if ch == '\t' {
            // Tab - move to next tab stop (every 8 chars)
            let tab_stop = ((current_x - x) / 8 + 1) * 8 + x;
            while current_x < tab_stop && current_x < x + width {
                buf.get_mut(current_x, y).set_symbol(" ");
                current_x += 1;
            }
        } else if ch.is_control() {
            // Skip other control characters
        } else {
            // Regular character
            let cell = buf.get_mut(current_x, y);
            cell.set_symbol(&ch.to_string());
            cell.set_style(current_style);
            current_x += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
        }
    }
}

/// Parse SGR (Select Graphic Rendition) parameters
fn parse_sgr(params: &str, mut style: Style) -> Style {
    if params.is_empty() {
        return Style::default();
    }

    let codes: Vec<u8> = params
        .split(';')
        .filter_map(|s| s.parse().ok())
        .collect();

    let mut i = 0;
    while i < codes.len() {
        match codes[i] {
            0 => style = Style::default(),
            1 => style = style.bold(),
            2 => style = style.dim(),
            3 => style = style.italic(),
            4 => style = style.underlined(),
            7 => style = style.reversed(),
            22 => style = style.not_bold().not_dim(),
            23 => style = style.not_italic(),
            24 => style = style.not_underlined(),
            27 => style = style.not_reversed(),

            // Foreground colors
            30 => style = style.fg(Color::Black),
            31 => style = style.fg(Color::Red),
            32 => style = style.fg(Color::Green),
            33 => style = style.fg(Color::Yellow),
            34 => style = style.fg(Color::Blue),
            35 => style = style.fg(Color::Magenta),
            36 => style = style.fg(Color::Cyan),
            37 => style = style.fg(Color::White),
            38 => {
                // Extended foreground color
                if i + 2 < codes.len() && codes[i + 1] == 5 {
                    style = style.fg(Color::Indexed(codes[i + 2]));
                    i += 2;
                } else if i + 4 < codes.len() && codes[i + 1] == 2 {
                    style = style.fg(Color::Rgb(codes[i + 2], codes[i + 3], codes[i + 4]));
                    i += 4;
                }
            }
            39 => style = style.fg(Color::Reset),

            // Bright foreground colors
            90 => style = style.fg(Color::DarkGray),
            91 => style = style.fg(Color::LightRed),
            92 => style = style.fg(Color::LightGreen),
            93 => style = style.fg(Color::LightYellow),
            94 => style = style.fg(Color::LightBlue),
            95 => style = style.fg(Color::LightMagenta),
            96 => style = style.fg(Color::LightCyan),
            97 => style = style.fg(Color::White),

            // Background colors
            40 => style = style.bg(Color::Black),
            41 => style = style.bg(Color::Red),
            42 => style = style.bg(Color::Green),
            43 => style = style.bg(Color::Yellow),
            44 => style = style.bg(Color::Blue),
            45 => style = style.bg(Color::Magenta),
            46 => style = style.bg(Color::Cyan),
            47 => style = style.bg(Color::White),
            48 => {
                // Extended background color
                if i + 2 < codes.len() && codes[i + 1] == 5 {
                    style = style.bg(Color::Indexed(codes[i + 2]));
                    i += 2;
                } else if i + 4 < codes.len() && codes[i + 1] == 2 {
                    style = style.bg(Color::Rgb(codes[i + 2], codes[i + 3], codes[i + 4]));
                    i += 4;
                }
            }
            49 => style = style.bg(Color::Reset),

            // Bright background colors
            100 => style = style.bg(Color::DarkGray),
            101 => style = style.bg(Color::LightRed),
            102 => style = style.bg(Color::LightGreen),
            103 => style = style.bg(Color::LightYellow),
            104 => style = style.bg(Color::LightBlue),
            105 => style = style.bg(Color::LightMagenta),
            106 => style = style.bg(Color::LightCyan),
            107 => style = style.bg(Color::White),

            _ => {}
        }
        i += 1;
    }

    style
}
