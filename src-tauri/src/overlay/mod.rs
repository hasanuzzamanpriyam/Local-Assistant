pub mod window;
pub mod shortcut;

use crate::overlay::window::OverlayWindow;
use crate::overlay::shortcut::GlobalShortcut;
use anyhow::Result;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize)]
pub struct OverlayUpdateEvent {
    pub step: u32,
    pub max_steps: u32,
    pub thought: Option<String>,
    pub action: Option<String>,
    pub elapsed_ms: u64,
}

pub struct OverlayManager {
    window: Arc<Mutex<OverlayWindow>>,
    shortcut: Arc<GlobalShortcut>,
    is_visible: Mutex<bool>,
    current_step: Mutex<u32>,
    max_steps: Mutex<u32>,
}

impl OverlayManager {
    pub fn new() -> Self {
        Self {
            window: Arc::new(Mutex::new(OverlayWindow::default())),
            shortcut: Arc::new(GlobalShortcut::new()),
            is_visible: Mutex::new(false),
            current_step: Mutex::new(0),
            max_steps: Mutex::new(10),
        }
    }

    pub fn set_window(&self, window: OverlayWindow) {
        *self.window.lock().unwrap() = window;
    }

    pub fn set_shortcut(&mut self, shortcut: GlobalShortcut) {
        self.shortcut = Arc::new(shortcut);
    }

    pub fn show(&self) -> Result<()> {
        self.window.lock().unwrap().show()?;
        *self.is_visible.lock().unwrap() = true;
        Ok(())
    }

    pub fn hide(&self) -> Result<()> {
        self.window.lock().unwrap().hide()?;
        *self.is_visible.lock().unwrap() = false;
        Ok(())
    }

    pub fn is_visible(&self) -> bool {
        *self.is_visible.lock().unwrap()
    }

    pub fn update(&self, step: u32, max_steps: u32, thought: Option<String>, action: Option<String>, elapsed_ms: u64) -> Result<()> {
        *self.current_step.lock().unwrap() = step;
        *self.max_steps.lock().unwrap() = max_steps;
        Ok(())
    }
}

impl Default for OverlayManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_manager_creates() {
        let manager = OverlayManager::new();
        assert!(!manager.is_visible());
    }
}
