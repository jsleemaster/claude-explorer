mod file_tree_widget;
mod terminal_widget;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use file_tree_widget::FileTreeWidget;
use terminal_widget::TerminalWidget;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Main layout: tree on right, terminal on left
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(100 - app.tree_width_percent),
            Constraint::Percentage(app.tree_width_percent),
        ])
        .split(size);

    // Terminal pane (left/main area)
    let terminal_area = chunks[0];
    let terminal_block = Block::default()
        .title(" Claude Code ")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let terminal_inner = terminal_block.inner(terminal_area);
    frame.render_widget(terminal_block, terminal_area);

    // Resize PTY to match terminal area
    app.terminal
        .resize(terminal_inner.width, terminal_inner.height);

    // Store terminal area for mouse drag routing
    app.terminal_area = Some(terminal_inner);

    let terminal_widget = TerminalWidget::new(&app.terminal, app.selection.as_ref());
    frame.render_widget(terminal_widget, terminal_inner);

    // Set hardware blinking cursor position (terminal always focused)
    {
        let vterm = app.terminal.vterm_lock();
        let cursor = vterm.cursor();
        if cursor.visible {
            let cx =
                terminal_inner.x + (cursor.x as u16).min(terminal_inner.width.saturating_sub(1));
            let cy =
                terminal_inner.y + (cursor.y as u16).min(terminal_inner.height.saturating_sub(1));
            if cx < terminal_inner.x + terminal_inner.width
                && cy < terminal_inner.y + terminal_inner.height
            {
                frame.set_cursor_position((cx, cy));
            }
        }
    }

    // File tree pane (right side)
    let tree_area = chunks[1];

    let tree_title = format!(
        " {} ",
        app.tree
            .root_path()
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| app.tree.root_path().to_string_lossy().to_string())
    );

    let tree_block = Block::default()
        .title(tree_title)
        .title_style(Style::default().fg(Color::Yellow).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let tree_inner = tree_block.inner(tree_area);
    frame.render_widget(tree_block, tree_area);

    // Store tree area for mouse scroll routing
    app.tree_area = Some(tree_inner);

    if app.tree_loading {
        let loading =
            Paragraph::new("  Scanning files...").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(loading, tree_inner);
    } else {
        // Auto-scroll to keep CWD visible — only when CWD actually changes
        let visible_height = tree_inner.height as usize;
        let cwd = app.terminal.cwd();
        let cwd_changed = app
            .last_auto_scroll_cwd
            .as_ref()
            .is_none_or(|last| last.as_path() != cwd);

        if cwd_changed {
            let cwd_index = app
                .tree
                .nodes()
                .iter()
                .position(|n| n.is_dir && n.path == cwd);

            if let Some(idx) = cwd_index {
                let mut offset = app.tree.offset();
                if idx >= offset + visible_height {
                    offset = idx - visible_height + 1;
                } else if idx < offset {
                    offset = idx;
                }
                app.tree.set_offset(offset);
            }
            app.last_auto_scroll_cwd = Some(cwd.to_path_buf());
        }

        // Render file tree
        let file_tree_widget = FileTreeWidget::new(&app.tree, Some(app.terminal.cwd()));
        frame.render_stateful_widget(
            file_tree_widget,
            tree_inner,
            &mut FileTreeWidgetState {
                offset: app.tree.offset(),
            },
        );
    }
}

pub struct FileTreeWidgetState {
    pub offset: usize,
}
