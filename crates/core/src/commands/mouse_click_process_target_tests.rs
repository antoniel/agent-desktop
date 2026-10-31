use super::{MouseClickArgs, execute};
use crate::commands::process_target_test_support::TwoInstances;
use crate::context::CommandContext;
use crate::{MouseButton, MouseEventKind};

fn args(window_id: Option<&str>, process_target: bool) -> MouseClickArgs {
    MouseClickArgs {
        x: 120.0,
        y: 80.0,
        button: MouseButton::Left,
        count: 2,
        modifiers: Vec::new(),
        window_id: window_id.map(str::to_owned),
        process_target,
    }
}

#[test]
fn process_target_click_reaches_only_the_named_window_without_headed() {
    let adapter = TwoInstances::default();
    let response = execute(
        args(Some("w-42"), true),
        &adapter,
        &CommandContext::default(),
    )
    .unwrap();
    let (window, event) = adapter.clicked.lock().unwrap().take().unwrap();
    assert_eq!(window, "w-42");
    assert!(matches!(event.kind, MouseEventKind::Click { count: 2 }));
    assert_eq!((event.point.x, event.point.y), (120.0, 80.0));
    assert_eq!(response["window_id"], "w-42");
    assert_eq!(response["process_target"], true);
}

#[test]
fn process_target_click_requires_a_window() {
    let adapter = TwoInstances::default();
    let error = execute(args(None, true), &adapter, &CommandContext::default()).unwrap_err();
    assert_eq!(error.code(), "INVALID_ARGS");
    assert!(adapter.clicked.lock().unwrap().is_none());
}

#[test]
fn process_target_click_refuses_headed_before_delivery() {
    let adapter = TwoInstances::default();
    let error = execute(
        args(Some("w-42"), true),
        &adapter,
        &CommandContext::default().with_headed(true),
    )
    .unwrap_err();
    assert_eq!(error.code(), "INVALID_ARGS");
    assert!(adapter.clicked.lock().unwrap().is_none());
}

#[test]
fn process_target_click_refuses_an_unknown_window_before_delivery() {
    let adapter = TwoInstances::default();
    let error = execute(
        args(Some("w-7"), true),
        &adapter,
        &CommandContext::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "WINDOW_NOT_FOUND");
    assert!(adapter.clicked.lock().unwrap().is_none());
}

#[test]
fn window_id_without_process_target_is_refused() {
    let adapter = TwoInstances::default();
    let error = execute(
        args(Some("w-42"), false),
        &adapter,
        &CommandContext::default().with_headed(true),
    )
    .unwrap_err();
    assert_eq!(error.code(), "INVALID_ARGS");
    assert!(adapter.clicked.lock().unwrap().is_none());
}
