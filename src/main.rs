mod ai;
mod app;
mod config;
mod document;
mod export;
mod image_loader;
mod ipc;
mod renderer;
mod syntax;
mod theme;
mod theme_loader;
mod toc;
mod vim;
mod watcher;

use app::MdeaderApp;
use clap::Parser;
use ipc::{IpcClient, IpcCommand};
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

    /// Theme to use (GitHubDark, GitHubLight, CatppuccinMocha, CatppuccinLatte, Dracula, Nord, SolarizedDark, SolarizedLight, or custom theme id)
    #[arg(short, long)]
    theme: Option<String>,

    /// Initial zoom factor (e.g. 1.0, 1.25)
    #[arg(short, long)]
    zoom: Option<f32>,

    /// Enable modal Vim keybinding navigation
    #[arg(long)]
    vim: bool,

    /// Remote command: open file in running mdeader instance
    #[arg(long = "remote-open", value_name = "FILE")]
    remote_open: Option<PathBuf>,

    /// Remote command: jump to heading in running mdeader instance
    #[arg(long = "remote-heading", value_name = "HEADING")]
    remote_heading: Option<String>,

    /// Remote command: reload file in running mdeader instance
    #[arg(long = "remote-reload")]
    remote_reload: bool,

    /// Remote command: send AI prompt in running mdeader instance
    #[arg(long = "remote-ai", value_name = "PROMPT")]
    remote_ai: Option<String>,

    /// Remote command: check if mdeader is running
    #[arg(long = "remote-ping")]
    remote_ping: bool,

    /// Remote IPC port (default: 19842)
    #[arg(long = "remote-port")]
    remote_port: Option<u16>,
}

fn main() -> eframe::Result<()> {
    let args = Cli::parse();

    // Handle remote IPC commands if specified
    let port = args.remote_port.unwrap_or(19842);
    if let Some(ref path) = args.remote_open {
        let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
        match IpcClient::send(port, &IpcCommand::OpenFile(abs_path)) {
            Ok(_) => {
                println!("[OK] Sent open command to running mdeader instance on port {}", port);
                return Ok(());
            }
            Err(e) => {
                eprintln!("[ERROR] {}", e);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref heading) = args.remote_heading {
        match IpcClient::send(port, &IpcCommand::JumpToHeading(heading.clone())) {
            Ok(_) => {
                println!("[OK] Sent jump to heading command to running mdeader instance on port {}", port);
                return Ok(());
            }
            Err(e) => {
                eprintln!("[ERROR] {}", e);
                std::process::exit(1);
            }
        }
    }

    if args.remote_reload {
        match IpcClient::send(port, &IpcCommand::Reload) {
            Ok(_) => {
                println!("[OK] Sent reload command to running mdeader instance on port {}", port);
                return Ok(());
            }
            Err(e) => {
                eprintln!("[ERROR] {}", e);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref prompt) = args.remote_ai {
        match IpcClient::send(port, &IpcCommand::AskAi(prompt.clone())) {
            Ok(_) => {
                println!("[OK] Sent AI prompt to running mdeader instance on port {}", port);
                return Ok(());
            }
            Err(e) => {
                eprintln!("[ERROR] {}", e);
                std::process::exit(1);
            }
        }
    }

    if args.remote_ping {
        match IpcClient::send(port, &IpcCommand::Ping) {
            Ok(msg) => {
                println!("[OK] Running mdeader instance replied: {}", msg);
                return Ok(());
            }
            Err(e) => {
                eprintln!("[ERROR] {}", e);
                std::process::exit(1);
            }
        }
    }

    let initial_title = if let Some(ref path) = args.file {
        format!(
            "mdeader - {}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("document")
        )
    } else {
        "mdeader".to_string()
    };

    let mut viewport = egui::ViewportBuilder::default()
        .with_app_id("mdeader")
        .with_inner_size([1150.0, 780.0])
        .with_min_inner_size([650.0, 450.0])
        .with_title(initial_title);

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let file = args.file;
    let theme = args.theme;
    let zoom = args.zoom;
    let vim = args.vim;

    eframe::run_native(
        "mdeader",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(MdeaderApp::new(cc, file, theme, zoom, vim)))
        }),
    )
}

fn load_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../assets/icon.png");
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}
