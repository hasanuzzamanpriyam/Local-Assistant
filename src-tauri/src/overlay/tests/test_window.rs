use crate::overlay::window::OverlayWindow;

#[test]
fn test_overlay_window_default_state() {
    let window = OverlayWindow::default();
    assert!(!window.is_visible());
}
