use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

pub struct HelpPopup;

impl HelpPopup {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for HelpPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the background
        Clear.render(area, buf);

        let help_text = r#"
╭──────────────────────────────────────────────────────╮
│              Claude Explorer - Help                  │
╰──────────────────────────────────────────────────────╯

 Global
 ──────────────────────────────────────────────────────
  Tab          Switch between Tree and Terminal panes
  Ctrl+T       Focus Tree pane
  Ctrl+Q       Quit application
  F1 / ?       Toggle this help

 File Tree Pane
 ──────────────────────────────────────────────────────
  j / ↓        Move down
  k / ↑        Move up
  g / Home     Go to first item
  G / End      Go to last item
  PgUp/PgDn    Page up/down

  Enter / l    Open directory / Insert file path
  h / ←        Collapse or go to parent
  Space        Toggle expand/collapse

  /            Start search
  n            Next search result
  N            Previous search result

  .            Toggle hidden files
  r / F5       Refresh tree

 Terminal Pane
 ──────────────────────────────────────────────────────
  All keys are passed to Claude Code
  Ctrl+C       Send interrupt to Claude Code

 Tips
 ──────────────────────────────────────────────────────
  • Press Enter on a file to insert @path in terminal
  • Press Enter on expanded dir to cd into it
  • Use Tab to quickly switch focus

                    Press any key to close
"#;

        let block = Block::default()
            .title(" Help ")
            .title_style(Style::default().fg(Color::Cyan).bold())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Rgb(30, 30, 40)));

        let paragraph = Paragraph::new(help_text)
            .block(block)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}
