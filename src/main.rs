use crate::common::common::File;
use crate::epub::Epub;
use crate::{app::App, models::epub};
use clap::Parser;
use crossterm::terminal;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::path::Path;
mod app;
pub mod common;
mod models;
#[derive(Parser)]
struct Cli {
    file: String,
}

fn main() {
    let path = Path::new(
        "/home/duc/Documents/クールな女神様と一緒に住んだら、甘やかしすぎてポンコツにしてしまった件について1 (HJ文庫).epub",
    );

    let epub_file: Epub = Epub::default();
    let result = epub_file.unzip(path);
    println!("exists: {}", path.exists());
    println!("result: {:#?}", result);
}
