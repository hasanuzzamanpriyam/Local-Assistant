use crate::sidecar::SidecarManager;

#[test]
fn test_sidecar_manager_creates() {
    let manager = SidecarManager::new();
    assert!(!manager.is_running());
}

#[test]
fn test_sidecar_manager_status_default() {
    let manager = SidecarManager::new();
    let status = manager.status();
    assert_eq!(status.running, false);
    assert_eq!(status.ready, false);
}
