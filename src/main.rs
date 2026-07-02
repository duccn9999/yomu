use crate::epub::epub::load;
use clap::Parser;
use crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use std::collections::HashMap;
use std::path::Path;
mod app;
mod epub;
#[derive(Parser)]
struct Cli {
    file: String,
}

enum Focus {
    Toc,
    Content,
}
struct AppState {
    epub_file: HashMap<String, Vec<String>>,
    keys: Vec<String>,
    selected_index: usize,
    scroll: u16,
    list_state: ListState,
    focus: Focus,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let path = Path::new(
        "/home/duc/Documents/epubs/Nữ Thần Tượng Nhà Bên Trót Phải Lòng Cơm Tôi Nấu_VN.epub",
    );
    let epub_file = load(path);
    let keys: Vec<String> = epub_file.keys().cloned().collect();
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    let mut state = AppState {
        epub_file,
        keys,
        selected_index: 0,
        scroll: 0,
        list_state,
        focus: Focus::Toc,
    };

    loop {
        terminal.draw(|f| render(f, &mut state))?;
        if let Event::Key(key) = crossterm::event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Tab => {
                    state.focus = match state.focus {
                        Focus::Toc => Focus::Content,
                        Focus::Content => Focus::Toc,
                    };
                }
                KeyCode::Down | KeyCode::Char('j') => match state.focus {
                    Focus::Toc => {
                        let max = state.keys.len().saturating_sub(1);
                        if state.selected_index < max {
                            state.selected_index += 1;
                            state.list_state.select(Some(state.selected_index));
                            state.scroll = 0;
                        }
                    }
                    Focus::Content => {
                        state.scroll = state.scroll.saturating_add(1);
                    }
                },
                KeyCode::Up | KeyCode::Char('k') => match state.focus {
                    Focus::Toc => {
                        if state.selected_index > 0 {
                            state.selected_index -= 1;
                            state.list_state.select(Some(state.selected_index));
                            state.scroll = 0;
                        }
                    }
                    Focus::Content => {
                        state.scroll = state.scroll.saturating_sub(1);
                    }
                },
                _ => {}
            }
        }
    }
    Ok(())
}
fn render(frame: &mut Frame, state: &mut AppState) {
    let toc_items: Vec<ListItem> = state
        .keys
        .iter()
        .map(|k| ListItem::new(k.as_str()))
        .collect();

    let toc_block = Block::new()
        .borders(Borders::ALL)
        .title("Table of content")
        .border_style(match state.focus {
            Focus::Toc => Style::default().fg(Color::Yellow),
            Focus::Content => Style::default(),
        });

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(15), Constraint::Percentage(85)])
        .split(frame.area());

    /* render content */
    let content_block = Block::new()
        .borders(Borders::ALL)
        .title("Title")
        .border_style(match state.focus {
            Focus::Content => Style::default().fg(Color::Yellow),
            Focus::Toc => Style::default(),
        });
    let lines: Vec<Line> = state
        .keys
        .get(state.selected_index)
        .and_then(|k| state.epub_file.get(k))
        .map(|v| v.iter().map(|l| Line::from(l.as_str())).collect())
        .unwrap_or_default();

    frame.render_stateful_widget(
        List::new(toc_items).block(toc_block).highlight_symbol("> "),
        horizontal[0],
        &mut state.list_state,
    );
    frame.render_widget(
        Paragraph::new(lines)
        .block(content_block)
        .scroll((state.scroll, 0))
        .wrap(Wrap { trim: true }),
        horizontal[1],
    );
}
