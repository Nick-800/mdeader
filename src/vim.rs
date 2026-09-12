use egui::{Context, Key};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimAction {
    ScrollDownLine,
    ScrollUpLine,
    ScrollDownHalfPage,
    ScrollUpHalfPage,
    ScrollTop,
    ScrollBottom,
    FocusSearch,
    SearchNext,
    SearchPrev,
    ToggleToc,
    ToggleAi,
    ToggleZen,
    ToggleSlideMode,
    Reload,
    OpenFile,
    CopyRaw,
    QuitOrEscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimModeState {
    Normal,
    Find,
}

#[derive(Debug, Clone)]
pub struct VimController {
    pub enabled: bool,
    pub mode: VimModeState,
    pub pending_keys: String,
}

impl Default for VimController {
    fn default() -> Self {
        Self::new(false)
    }
}

impl VimController {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            mode: VimModeState::Normal,
            pending_keys: String::new(),
        }
    }

    /// Handles keyboard events according to Vim navigation rules.
    pub fn handle_input(&mut self, ctx: &Context) -> Option<VimAction> {
        if !self.enabled {
            return None;
        }

        let has_focus = ctx.wants_keyboard_input();
        let input = ctx.input(|i| i.clone());

        // Escape always resets focus and returns to Normal mode
        if input.key_pressed(Key::Escape) {
            self.mode = VimModeState::Normal;
            self.pending_keys.clear();
            ctx.memory_mut(|m| {
                if let Some(id) = m.focused() {
                    m.surrender_focus(id);
                }
            });
            return Some(VimAction::QuitOrEscape);
        }

        // If an input textfield currently has keyboard focus, let user type
        if has_focus {
            return None;
        }

        // Only process navigation in Normal mode
        if self.mode != VimModeState::Normal {
            return None;
        }

        let has_cmd = input.modifiers.command || input.modifiers.ctrl;

        // Navigation key actions: Ctrl+D / Ctrl+U
        if has_cmd && input.key_pressed(Key::D) {
            return Some(VimAction::ScrollDownHalfPage);
        }
        if has_cmd && input.key_pressed(Key::U) {
            return Some(VimAction::ScrollUpHalfPage);
        }

        // Only handle unmodified single letter keys if no Command / Ctrl is pressed
        if !has_cmd {
            if input.key_pressed(Key::J) || input.key_pressed(Key::ArrowDown) {
                return Some(VimAction::ScrollDownLine);
            }
            if input.key_pressed(Key::K) || input.key_pressed(Key::ArrowUp) {
                return Some(VimAction::ScrollUpLine);
            }

            // 'gg' or 'G'
            if input.modifiers.shift && input.key_pressed(Key::G) {
                self.pending_keys.clear();
                return Some(VimAction::ScrollBottom);
            }

            if input.key_pressed(Key::G) {
                if self.pending_keys == "g" {
                    self.pending_keys.clear();
                    return Some(VimAction::ScrollTop);
                } else {
                    self.pending_keys = "g".to_string();
                    return None;
                }
            }

            // 'yy' copy
            if input.key_pressed(Key::Y) {
                if self.pending_keys == "y" {
                    self.pending_keys.clear();
                    return Some(VimAction::CopyRaw);
                } else {
                    self.pending_keys = "y".to_string();
                    return None;
                }
            }

            // Reset pending keys if any other key was pressed
            if !self.pending_keys.is_empty() {
                self.pending_keys.clear();
            }

            // Search: '/'
            if input.key_pressed(Key::Slash) {
                self.mode = VimModeState::Find;
                return Some(VimAction::FocusSearch);
            }

            // Next / Prev search: 'n' / 'N'
            if input.modifiers.shift && input.key_pressed(Key::N) {
                return Some(VimAction::SearchPrev);
            }
            if input.key_pressed(Key::N) {
                return Some(VimAction::SearchNext);
            }

            // Toggle TOC: 'b'
            if input.key_pressed(Key::B) {
                return Some(VimAction::ToggleToc);
            }

            // Toggle AI: 'a'
            if input.key_pressed(Key::A) {
                return Some(VimAction::ToggleAi);
            }

            // Toggle Zen Mode: 'z'
            if input.key_pressed(Key::Z) {
                return Some(VimAction::ToggleZen);
            }

            // Toggle Slide Presentation Mode: 'p'
            if input.key_pressed(Key::P) {
                return Some(VimAction::ToggleSlideMode);
            }

            // Reload: 'r'
            if input.key_pressed(Key::R) {
                return Some(VimAction::Reload);
            }

            // Open: 'o'
            if input.key_pressed(Key::O) {
                return Some(VimAction::OpenFile);
            }
        }

        None
    }

    pub fn status_badge(&self) -> &'static str {
        match self.mode {
            VimModeState::Normal => "[VIM: NORMAL]",
            VimModeState::Find => "[VIM: FIND]",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_controller_default() {
        let vim = VimController::new(true);
        assert!(vim.enabled);
        assert_eq!(vim.mode, VimModeState::Normal);
        assert_eq!(vim.status_badge(), "[VIM: NORMAL]");
    }
}
