use crate::overlay::shortcut::GlobalShortcut;

#[test]
fn test_global_shortcut_creates() {
    let shortcut = GlobalShortcut::new();
    assert!(!shortcut.is_registered());
}
