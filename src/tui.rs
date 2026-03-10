use std::io::{self, Stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{execute, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Margin;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use unicode_width::UnicodeWidthStr;

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

pub fn run(markdown: &str) -> Result<()> {
    let mut guard = TerminalGuard::new()?;
    let mut scroll = 0u16;

    loop {
        let (tw, th) = terminal::size()?;
        let max_scroll = compute_max_scroll(markdown, tw, th);
        scroll = scroll.min(max_scroll);

        guard
            .terminal
            .draw(|frame| render(frame, markdown, scroll))?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Char('q') => break,
            KeyCode::Down | KeyCode::Char('j') => {
                scroll = scroll.saturating_add(1).min(max_scroll);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                scroll = scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                scroll = scroll.saturating_add(10).min(max_scroll);
            }
            KeyCode::PageUp => {
                scroll = scroll.saturating_sub(10);
            }
            KeyCode::Home => {
                scroll = 0;
            }
            KeyCode::End => {
                scroll = max_scroll;
            }
            _ => {}
        }
    }

    Ok(())
}

fn render(frame: &mut Frame<'_>, markdown: &str, scroll: u16) {
    let area = frame.area();
    let block = Block::default()
        .title(Line::from(" atv ").style(Style::default().add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL);

    let inner = block.inner(area).inner(Margin {
        vertical: 0,
        horizontal: 1,
    });
    frame.render_widget(block, area);

    let paragraph = Paragraph::new(markdown)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

fn compute_max_scroll(markdown: &str, term_width: u16, term_height: u16) -> u16 {
    let viewport_height = term_height.saturating_sub(2) as usize;
    let content_width = term_width.saturating_sub(4).max(1) as usize;
    if viewport_height == 0 {
        return 0;
    }

    let visual_lines = markdown
        .split('\n')
        .map(|line| {
            let w = UnicodeWidthStr::width(line);
            usize::max(1, w.div_ceil(content_width))
        })
        .sum::<usize>();

    visual_lines
        .saturating_sub(viewport_height)
        .min(u16::MAX as usize) as u16
}

#[cfg(test)]
mod tests {
    use super::compute_max_scroll;

    #[test]
    fn clamp_when_content_short() {
        assert_eq!(compute_max_scroll("a\nb", 120, 40), 0);
    }

    #[test]
    fn scroll_when_content_long() {
        let text = (0..200)
            .map(|i| format!("line-{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(compute_max_scroll(&text, 80, 20) > 0);
    }
}
