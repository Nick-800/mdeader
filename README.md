# mdeader

<p align="center">
  <img src="assets/logo.png" alt="mdeader logo" width="180" />
</p>

A standalone, fast, and customizable native Markdown reader built in Rust using [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe) / [`egui`](https://github.com/emilk/egui).

`mdeader` provides a smooth, instantaneous desktop reading experience for Markdown documents with zero web engine / Electron overhead.

---

## Features

- **Native Performance**: Instant 60+ FPS rendering powered by `egui` and `pulldown-cmark`, using ~30MB memory.
- **8 Built-in Themes**:
  - `GitHub Dark` (default)
  - `GitHub Light`
  - `Catppuccin Mocha`
  - `Catppuccin Latte`
  - `Dracula`
  - `Nord`
  - `Solarized Dark`
  - `Solarized Light`
- **Table of Contents (TOC) Sidebar**: Hierarchical document outline (H1–H6) with clickable jump-to navigation and real-time heading filter.
- **In-Document Search**: Interactive find bar (`Ctrl+F`) with match count, previous/next cycling, and highlighted search terms.
- **Live Auto-Reload**: Watches opened files with debouncing (`notify-debouncer-mini`) and seamlessly updates content when edited externally.
- **Syntax Highlighting**: Full code block highlighting via `syntect` with a one-click "Copy" button.
- **GitHub-Flavored Markdown Alerts**: Native rendering for `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, and `[!CAUTION]` with themed borders and backgrounds.
- **AI Reading Assistant**: Collapsible right drawer (`Ctrl+Shift+A` or `Ctrl+I`) for document Q&A, instant chapter summarization, and checklist/action-item extraction.
- **Local-First & Cloud AI Backends**: Works out of the box with local Ollama instances (`llama3.2`, `mistral`), Google Gemini, or custom OpenAI-compatible endpoints.
- **Smart Code Block Explanations**: One-click `[Explain]` button directly inside syntax-highlighted code block headers to query the AI assistant.
- **Agent & Terminal IPC Bridge**: Remote control socket interface allowing AI agents (such as Antigravity) and shell scripts to open files, jump to headings, trigger reloads, and submit prompts into running `mdeader` instances.
- **Document Statistics**: Live word count, character count, line count, and estimated reading time.
- **Self-Contained HTML Export**: Export formatted Markdown documents to standalone HTML with embedded styling matching your active theme.
- **Image Support**: Renders local relative and absolute images with automatic texture caching.
- **Interactive Links**: Opens web links in your default browser and switches to local `.md` files directly inside `mdeader`.
- **Zen Distraction-Free Mode**: Toggle full-window focused reading (`F11` or `z`) hiding all toolbars, panels, and sidebars.
- **Slide Presentation Mode**: Convert markdown documents into presentation decks (`F5` or `p`), splitting slides at `---` horizontal rules with previous/next slide navigation and counter.
- **Modal Vim Navigation Mode**: Full keyboard-driven navigation (`--vim` or toggle via status bar): `j`/`k` scrolling, `Ctrl+u`/`Ctrl+d` half-page jumps, `gg`/`G` top/bottom, `/` search, `n`/`N` next/prev match, `b` TOC toggle, `a` AI assistant toggle, `z` Zen mode, `p` slide presentation, `yy` copy markdown to clipboard.
- **Interactive Task Lists**: Clickable task checkboxes (`- [ ]` / `- [x]`) that toggle state in the viewer and immediately persist back to the markdown file on disk.
- **Custom TOML Theme Loader**: Drop any custom palette into `~/.config/mdeader/themes/*.toml` with full support for UI colors, borders, and syntax highlighting themes.
- **Centered Typography & Max Width**: Configurable `max_content_width` (default 860px) keeps lines at optimal reading length on widescreen and ultrawide monitors.
- **Recent Files & Drag-and-Drop**: Easily access your recently opened documents or drop any `.md` file onto the window.

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

### Remote Control & AI Agent IPC

Control an already running `mdeader` desktop instance from terminal commands or AI coding agents:

```bash
# Open a file in the active mdeader window
mdeader --remote-open /path/to/DOCUMENT.md

# Navigate to a specific heading section
mdeader --remote-heading "Architecture"

# Reload the currently viewed document
mdeader --remote-reload

# Ask the AI assistant to summarize or answer a question
mdeader --remote-ai "Summarize the key architectural changes"

# Verify if a mdeader instance is listening
mdeader --remote-ping
```

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| **`Ctrl + O`** | Open file dialog |
| **`Ctrl + F`** | Toggle find/search bar |
| **`Esc`** | Close find bar / exit Zen or Slide mode |
| **`F11`** | Toggle Zen distraction-free mode |
| **`F5`** | Toggle slide presentation deck mode |
| **`Left` / `Right`** | Previous / Next slide (in presentation mode) |
| **`Ctrl + B`** | Toggle Table of Contents outline sidebar |
| **`Ctrl + Shift + A`** / **`Ctrl + I`** | Toggle AI Assistant drawer |
| **`Ctrl + R`** | Force reload current document |
| **`Ctrl + =` / `Ctrl + +`** | Zoom in |
| **`Ctrl + -`** | Zoom out |
| **`Ctrl + 0`** | Reset zoom (100%) |
| **`Ctrl + E`** | Export document as standalone HTML |

### Vim Mode Keybindings

When Vim mode is enabled (`--vim` or in config):

| Key | Action |
| :--- | :--- |
| **`j` / `k`** | Scroll down / up |
| **`Ctrl + d` / `Ctrl + u`** | Half-page down / up |
| **`g g` / `G`** | Jump to top / bottom |
| **`/`** | Open search bar |
| **`n` / `N`** | Next / previous search match |
| **`b`** | Toggle Table of Contents sidebar |
| **`a`** | Toggle AI Assistant drawer |
| **`z`** | Toggle Zen distraction-free mode |
| **`p`** | Toggle Slide presentation mode |
| **`r`** | Reload current file |
| **`o`** | Open file dialog |
| **`y y`** | Copy raw document text to clipboard |
| **`Esc`** | Cancel search / return to normal mode |

---

## Configuration

Preferences are stored in `~/.config/mdeader/config.toml` and automatically updated when settings are changed in the app:

```toml
theme = "GitHubDark"
font_size = 15.0
line_spacing = 1.4
max_content_width = 860.0
vim_mode = false
show_toc = true
toc_width = 240.0
watch_mode = true
zoom = 1.0
show_ai = false
ai_width = 340.0
ipc_enabled = true
ipc_port = 19842
recent_files = [ ... ]

[ai]
provider = "ollama"
endpoint = "http://localhost:11434"
model = "llama3.2"
# api_key = "..."

[keybindings]
open_file = "Ctrl+O"
reload = "Ctrl+R"
find = "Ctrl+F"
toggle_toc = "Ctrl+B"
toggle_ai = "Ctrl+Shift+A"
toggle_zen = "F11"
toggle_slides = "F5"
export_html = "Ctrl+E"
zoom_in = "Ctrl+Plus"
zoom_out = "Ctrl+Minus"
zoom_reset = "Ctrl+0"
scroll_top = "Home"
scroll_bottom = "End"
escape = "Escape"

[keybindings.vim]
scroll_down = "j"
scroll_up = "k"
scroll_down_half = "Ctrl+d"
scroll_up_half = "Ctrl+u"
scroll_top = "gg"
scroll_bottom = "G"
find = "/"
next_match = "n"
prev_match = "N"
toggle_toc = "b"
toggle_ai = "a"
toggle_zen = "z"
toggle_slides = "p"
reload = "r"
open_file = "o"
copy_raw = "yy"
```

---

## License

Licensed under the MIT License.
