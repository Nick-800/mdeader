#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum VimModeState {
    Normal,
    Find,
}

#[derive(Debug, Clone)]
pub struct VimController {
    pub enabled: bool,
    pub mode: VimModeState,
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
        }
    }

    pub fn status_badge(&self) -> &'static str {
        if !self.enabled {
            return "[VIM: OFF]";
        }
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

        let vim_off = VimController::new(false);
        assert_eq!(vim_off.status_badge(), "[VIM: OFF]");
    }
}
