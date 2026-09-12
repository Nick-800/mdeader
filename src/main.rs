mod app;
mod config;
mod document;
mod export;
mod image_loader;
mod renderer;
mod syntax;
mod theme;
mod toc;
mod watcher;

use app::MdeaderApp;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "mdeader",
    about = "Standalone, fast, and customizable Markdown reader in Rust",
    version
)]
struct Cli {
    /// Path to markdown file to open
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Theme to use (GitHubDark, GitHubLight, CatppuccinMocha, CatppuccinLatte, Dracula, Nord, SolarizedDark, SolarizedLight)
    #[arg(short, long)]
    theme: Option<String>,

    /// Initial zoom factor (e.g. 1.0, 1.25)
    #[arg(short, long)]
    zoom: Option<f32>,
}

fn main() -> eframe::Result<()> {
    let args = Cli::parse();

    let initial_title = if let Some(ref path) = args.file {
        format!(
            "mdeader - {}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("document")
        )
    } else {
        "mdeader".to_string()
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1150.0, 780.0])
            .with_min_inner_size([650.0, 450.0])
            .with_title(initial_title),
        ..Default::default()
    };

    let file = args.file;
    let theme = args.theme;
    let zoom = args.zoom;

    eframe::run_native(
        "mdeader",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(MdeaderApp::new(cc, file, theme, zoom)))
        }),
    )
}
