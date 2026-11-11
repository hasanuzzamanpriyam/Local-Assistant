use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutManager};
use tauri::{AppHandle, Manager};
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub struct GlobalShortcut {
    handler: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>,
    is_registered: Arc<Mutex<bool>>,
}

impl GlobalShortcut {
    pub fn new() -> Self {
        Self {
            handler: Arc::new(Mutex::new(None)),
            is_registered: Arc::new(Mutex::new(false)),
        }
    }

    pub fn is_registered(&self) -> bool {
        *self.is_registered.lock().unwrap()
    }

    pub fn register(&self, app_handle: &AppHandle) -> Result<()> {
        let handler = self.handler.lock().unwrap().clone();
        let shortcut_manager = app_handle.global_shortcut();

        shortcut_manager.register("Ctrl+Alt+Esc", move |_event, _win| {
            if let Some(ref h) = handler {
                h();
            }
        })?;

        *self.is_registered.lock().unwrap() = true;
        Ok(())
    }

    pub fn unregister(&self) -> Result<()> {
        *self.is_registered.lock().unwrap() = false;
        Ok(())
    }

    pub fn set_handler(&self, handler: Box<dyn Fn() + Send + Sync>) {
        *self.handler.lock().unwrap() = Some(handler);
    }
}

impl Default for GlobalShortcut {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_shortcut_creates() {
        let shortcut = GlobalShortcut::new();
        assert!(!shortcut.is_registered());
    }
}
