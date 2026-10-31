use super::*;

#[test]
fn background_process_target_reaches_owner_without_focusing_window() {
    let adapter = TwoInstancesAdapter::default();
    let result = execute(
        PressArgs {
            combo: "cmd+k".into(),
            app: None,
            window_id: Some("w-42".into()),
            process_target: true,
            force: false,
        },
        &adapter,
        &CommandContext::default(),
    );
    let value = result.unwrap();
    assert_eq!(value["process_target"], true);
    assert_eq!(value["window_id"], "w-42");
    assert_eq!(*adapter.pressed_pid.lock().unwrap(), Some(42));
    assert!(adapter.focused_window.lock().unwrap().is_none());
}

#[test]
fn background_process_target_requires_a_window_identity() {
    let adapter = TwoInstancesAdapter::default();
    let result = execute(
        PressArgs {
            combo: "cmd+k".into(),
            app: Some("Electron".into()),
            window_id: None,
            process_target: true,
            force: false,
        },
        &adapter,
        &CommandContext::default(),
    );
    assert_eq!(result.unwrap_err().code(), "INVALID_ARGS");
    assert!(adapter.pressed_pid.lock().unwrap().is_none());
}

#[test]
fn background_process_target_rejects_missing_window_before_delivery() {
    let adapter = TwoInstancesAdapter::default();
    let result = execute(
        PressArgs {
            combo: "cmd+k".into(),
            app: None,
            window_id: Some("w-7".into()),
            process_target: true,
            force: false,
        },
        &adapter,
        &CommandContext::default(),
    );
    assert_eq!(result.unwrap_err().code(), "WINDOW_NOT_FOUND");
    assert!(adapter.pressed_pid.lock().unwrap().is_none());
}

#[test]
fn background_process_target_refuses_headed_policy_before_delivery() {
    let adapter = TwoInstancesAdapter::default();
    let result = execute(
        PressArgs {
            combo: "cmd+k".into(),
            app: None,
            window_id: Some("w-42".into()),
            process_target: true,
            force: false,
        },
        &adapter,
        &CommandContext::default().with_headed(true),
    );
    assert_eq!(result.unwrap_err().code(), "INVALID_ARGS");
    assert!(adapter.pressed_pid.lock().unwrap().is_none());
    assert!(adapter.focused_window.lock().unwrap().is_none());
}
