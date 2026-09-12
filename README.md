# mdeader

A standalone, fast, and customizable native Markdown reader built in Rust using [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe) / [`egui`](https://github.com/emilk/egui).

`mdeader` provides a smooth, instantaneous desktop reading experience for Markdown documents with zero web engine / Electron overhead.

---

## Features

- ⚡ **Native Performance**: Instant 60+ FPS rendering powered by `egui` and `pulldown-cmark`, using ~30MB memory.
- 🎨 **8 Built-in Themes**:
  - `GitHub Dark` (default)
  - `GitHub Light`
  - `Catppuccin Mocha`
  - `Catppuccin Latte`
  - `Dracula`
  - `Nord`
  - `Solarized Dark`
  - `Solarized Light`
- 📑 **Table of Contents (TOC) Sidebar**: Hierarchical document outline (H1–H6) with clickable jump-to navigation and real-time heading filter.
- 🔍 **In-Document Search**: Interactive find bar (`Ctrl+F`) with match count, previous/next cycling, and highlighted search terms.
- 🔄 **Live Auto-Reload**: Watches opened files with debouncing (`notify-debouncer-mini`) and seamlessly updates content when edited externally.
- 💻 **Syntax Highlighting**: Full code block highlighting via `syntect` with a one-click "📋 Copy" button.
- 📊 **Document Statistics**: Live word count, character count, line count, and estimated reading time.
- 📤 **Self-Contained HTML Export**: Export formatted Markdown documents to standalone HTML with embedded styling matching your active theme.
- 🖼️ **Image Support**: Renders local relative and absolute images with automatic texture caching.
- 🔗 **Interactive Links**: Opens web links in your default browser and switches to local `.md` files directly inside `mdeader`.
- 🔍 **Zoom & Typography**: Dynamic font scaling (`Ctrl++` / `Ctrl+-` / `Ctrl+0`) and persistent preferences.
- 📁 **Recent Files & Drag-and-Drop**: Easily access your recently opened documents or drop any `.md` file onto the window.

---

## Installation

### From Source

Ensure you have Rust installed (1.75+):

```bash
git clone https://github.com/Nick-800/mdeader.git
cd mdeader
cargo install --path .
```

---

## Usage

```bash
# Open an empty window or recent file
mdeader

# Open a specific Markdown file
mdeader README.md

# Open with a specific theme
mdeader README.md --theme CatppuccinMocha

# Open with a custom zoom factor
mdeader README.md --zoom 1.2
```

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| **`Ctrl + O`** | Open file dialog |
| **`Ctrl + F`** | Toggle find/search bar |
| **`Esc`** | Close find bar |
| **`Ctrl + B`** | Toggle Table of Contents outline sidebar |
| **`Ctrl + R`** | Force reload current document |
| **`Ctrl + =` / `Ctrl + +`** | Zoom in |
| **`Ctrl + -`** | Zoom out |
| **`Ctrl + 0`** | Reset zoom (100%) |
| **`Ctrl + E`** | Export document as standalone HTML |

---

## Configuration

Preferences are stored in `~/.config/mdeader/config.toml` and automatically updated when settings are changed in the app:

```toml
theme = "GitHubDark"
font_size = 15.0
line_spacing = 1.4
show_toc = true
toc_width = 240.0
watch_mode = true
zoom = 1.0
recent_files = [ ... ]
```

---

## License

Licensed under the MIT License.
