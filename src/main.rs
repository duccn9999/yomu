use crate::common::common::File;
use crate::epub::Epub;
use crate::{app::App, models::epub};
use clap::Parser;
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};
use std::path::Path;
mod app;
pub mod common;
mod models;
#[derive(Parser)]
struct Cli {
    file: String,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.code == crossterm::event::KeyCode::Char('q') {
                break Ok(());
            }
        }
    }
}

fn render(frame: &mut Frame) {
    let path = Path::new("/home/duc/Documents/epubs/また同じ夢を見ていた - 住野よる.epub");
    let epub_file: Epub = Epub::default();
    let result = epub_file.unzip(path);

    let debug_text = format!("{:#?}", result);
    frame.render_widget(Paragraph::new(debug_text), frame.area());
}
