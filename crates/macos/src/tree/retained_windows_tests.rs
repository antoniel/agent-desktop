use super::*;
use agent_desktop_core::{ProcessId, WindowState};

fn window(id: &str, pid: u32, instance: &str) -> WindowInfo {
    WindowInfo {
        id: id.into(),
        title: String::new(),
        app: String::new(),
        pid: ProcessId::new(pid),
        process_instance: Some(instance.into()),
        bounds: None,
        state: WindowState::default(),
    }
}

fn element() -> AXElement {
    AXElement(unsafe { accessibility_sys::AXUIElementCreateApplication(std::process::id() as i32) })
}

#[test]
fn session_cache_separates_process_instance_and_window_identity() {
    let _session = crate::tree::resolve::retained::RetainedRefSession::start().unwrap();
    let item = element();
    save(&window("w-1", 1, "one"), &item);
    assert!(get("w-1", 1, "one").is_some());
    assert!(get("w-1", 1, "other").is_none());
    assert!(get("w-1", 2, "one").is_none());
    assert!(get("w-2", 1, "one").is_none());
}

#[test]
fn dropping_session_clears_retained_window_handles() {
    let session = crate::tree::resolve::retained::RetainedRefSession::start().unwrap();
    save(&window("w-1", 1, "one"), &element());
    assert!(get("w-1", 1, "one").is_some());
    drop(session);
    let _next = crate::tree::resolve::retained::RetainedRefSession::start().unwrap();
    assert!(get("w-1", 1, "one").is_none());
}

fn soon() -> std::time::Instant {
    std::time::Instant::now() + std::time::Duration::from_secs(2)
}

#[test]
fn handle_of_another_process_instance_is_dropped_not_reused() {
    let _session = crate::tree::resolve::retained::RetainedRefSession::start().unwrap();
    let pid = std::process::id();
    save(&window("w-1", pid, "not-this-instance"), &element());
    assert!(
        checked("w-1", pid, "not-this-instance", soon())
            .unwrap()
            .is_none()
    );
    assert!(get("w-1", pid, "not-this-instance").is_none());
}

/// The application element reports no window number: a stand-in for a window
/// element that was closed or replaced since it was retained.
#[test]
fn handle_that_no_longer_reports_the_window_is_dropped_not_reused() {
    let _session = crate::tree::resolve::retained::RetainedRefSession::start().unwrap();
    let pid = std::process::id();
    let instance = crate::system::process::process_identity::token_for_pid(pid as i32)
        .unwrap()
        .unwrap();
    save(&window("w-424242", pid, &instance), &element());
    assert!(
        checked("w-424242", pid, &instance, soon())
            .unwrap()
            .is_none()
    );
    assert!(get("w-424242", pid, &instance).is_none());
}

#[test]
fn handle_with_an_invalid_window_id_is_dropped() {
    let _session = crate::tree::resolve::retained::RetainedRefSession::start().unwrap();
    save(&window("not-a-window", 1, "one"), &element());
    assert!(checked("not-a-window", 1, "one", soon()).unwrap().is_none());
    assert!(get("not-a-window", 1, "one").is_none());
}
