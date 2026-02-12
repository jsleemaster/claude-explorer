use ratatui::{prelude::*, widgets::Widget};

use crate::app::Selection;
use crate::terminal::TerminalPane;

pub struct TerminalWidget<'a> {
    terminal: &'a TerminalPane,
    selection: Option<&'a Selection>,
}

impl<'a> TerminalWidget<'a> {
    pub fn new(terminal: &'a TerminalPane, selection: Option<&'a Selection>) -> Self {
        Self {
            terminal,
            selection,
        }
    }

    /// Check if a given (col, row) is within the selection range.
    fn is_selected(&self, col: u16, row: u16) -> bool {
        let sel = match self.selection {
            Some(s) => s,
            None => return false,
        };

        // Normalize so start <= end in reading order
        let (start, end) = if (sel.start.1, sel.start.0) <= (sel.end.1, sel.end.0) {
            (sel.start, sel.end)
        } else {
            (sel.end, sel.start)
        };

        if row < start.1 || row > end.1 {
            return false;
        }
        if row == start.1 && row == end.1 {
            return col >= start.0 && col <= end.0;
        }
        if row == start.1 {
            return col >= start.0;
        }
        if row == end.1 {
            return col <= end.0;
        }
        true
    }
}

impl<'a> Widget for TerminalWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vterm = self.terminal.vterm_lock();
        let grid = vterm.grid();
        let scrollback = vterm.scrollback();
        let scroll_offset = vterm.scroll_offset();

        if scroll_offset == 0 {
            // Normal mode: render the grid directly
            let rows_to_render = (area.height as usize).min(grid.len());
            let cols_to_render = (area.width as usize).min(vterm.cols());

            for row_idx in 0..rows_to_render {
                if let Some(row) = grid.get(row_idx) {
                    for (col_idx, cell) in row.iter().enumerate().take(cols_to_render) {
                        if cell.ch.is_empty() {
                            continue; // wide char continuation cell
                        }
                        let x = area.x + col_idx as u16;
                        let y = area.y + row_idx as u16;
                        if x < area.x + area.width && y < area.y + area.height {
                            if let Some(buf_cell) = buf.cell_mut((x, y)) {
                                buf_cell.set_symbol(&cell.ch);
                                let style = if self.is_selected(col_idx as u16, row_idx as u16) {
                                    cell.style.add_modifier(Modifier::REVERSED)
                                } else {
                                    cell.style
                                };
                                buf_cell.set_style(style);
                            }
                        }
                    }
                }
            }
        } else {
            // Scrollback mode: mix scrollback + grid
            let visible_height = area.height as usize;
            let cols_to_render = (area.width as usize).min(vterm.cols());
            let total_lines = scrollback.len() + grid.len();

            // scroll_offset is how many lines above the bottom of the grid we are
            let bottom = total_lines.saturating_sub(scroll_offset);
            let top = bottom.saturating_sub(visible_height);

            for (screen_row, line_idx) in (top..bottom).enumerate() {
                let row_data = if line_idx < scrollback.len() {
                    scrollback.get(line_idx)
                } else {
                    grid.get(line_idx - scrollback.len())
                };

                if let Some(row) = row_data {
                    for (col_idx, cell) in row.iter().enumerate().take(cols_to_render) {
                        if cell.ch.is_empty() {
                            continue; // wide char continuation cell
                        }
                        let x = area.x + col_idx as u16;
                        let y = area.y + screen_row as u16;
                        if x < area.x + area.width && y < area.y + area.height {
                            if let Some(buf_cell) = buf.cell_mut((x, y)) {
                                buf_cell.set_symbol(&cell.ch);
                                let style = if self.is_selected(col_idx as u16, screen_row as u16) {
                                    cell.style.add_modifier(Modifier::REVERSED)
                                } else {
                                    cell.style
                                };
                                buf_cell.set_style(style);
                            }
                        }
                    }
                }
            }
        }
    }
}
