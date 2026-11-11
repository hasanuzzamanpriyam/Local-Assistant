use tauri::{AppHandle, WebviewWindow, WebviewWindowBuilder};
use anyhow::Result;

pub struct OverlayWindow {
    window: Option<WebviewWindow>,
}

impl OverlayWindow {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let window = WebviewWindowBuilder::new(app_handle, "overlay", tauri::WebviewUrl::App("/overlay.html".into()))
            .title("Assistant Overlay")
            .inner_size(1920.0, 1080.0)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .focus()
            .visible(false)
            .build()?;

        Ok(Self { window: Some(window) })
    }

    pub fn show(&self) -> Result<()> {
        if let Some(ref window) = self.window {
            window.show()?;
            window.set_focus()?;
        }
        Ok(())
    }

    pub fn hide(&self) -> Result<()> {
        if let Some(ref window) = self.window {
            window.hide()?;
        }
        Ok(())
    }

    pub fn is_visible(&self) -> bool {
        self.window
            .as_ref()
            .map(|w| w.is_visible().unwrap_or(false))
            .unwrap_or(false)
    }
}

impl Default for OverlayWindow {
    fn default() -> Self {
        Self { window: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_window_default_state() {
        let window = OverlayWindow::default();
        assert!(!window.is_visible());
    }
}
