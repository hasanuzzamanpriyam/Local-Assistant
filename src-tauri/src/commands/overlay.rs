use tauri::{command, State};
use crate::overlay::OverlayManager;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OverlayShowRequest {
    pub x: f64,
    pub y: f64,
}

#[command]
pub fn overlay_show(manager: State<'_, OverlayManager>, request: OverlayShowRequest) -> Result<(), String> {
    let _ = request;
    manager.show().map_err(|e| e.to_string())
}

#[command]
pub fn overlay_hide(manager: State<'_, OverlayManager>) -> Result<(), String> {
    manager.hide().map_err(|e| e.to_string())
}
