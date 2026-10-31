use super::{DragArgs, DragEndpoint, WaitForScope, execute};
use crate::commands::process_target_test_support::TwoInstances;
use crate::context::CommandContext;

fn args(from: DragEndpoint, window_id: Option<&str>, process_target: bool) -> DragArgs {
    DragArgs {
        from,
        to: DragEndpoint {
            ref_id: None,
            xy: Some((300.0, 90.0)),
        },
        duration_ms: Some(50),
        drop_delay_ms: Some(10),
        timeout_ms: Some(5_000),
        wait_for_scope: WaitForScope::default(),
        window_id: window_id.map(str::to_owned),
        process_target,
    }
}

fn xy() -> DragEndpoint {
    DragEndpoint {
        ref_id: None,
        xy: Some((100.0, 90.0)),
    }
}

#[test]
fn process_target_drag_reaches_only_the_named_window_without_headed() {
    let adapter = TwoInstances::default();
    let response = execute(
        args(xy(), Some("w-41"), true),
        &adapter,
        &CommandContext::default(),
    )
    .unwrap();
    let (window, params) = adapter.dragged.lock().unwrap().take().unwrap();
    assert_eq!(window, "w-41");
    assert_eq!((params.from.x, params.from.y), (100.0, 90.0));
    assert_eq!((params.to.x, params.to.y), (300.0, 90.0));
    assert_eq!(params.duration_ms, Some(50));
    assert_eq!(response["window_id"], "w-41");
    assert_eq!(response["process_target"], true);
}

#[test]
fn process_target_drag_takes_coordinates_not_refs() {
    let adapter = TwoInstances::default();
    let from = DragEndpoint {
        ref_id: Some("@e1".into()),
        xy: None,
    };
    let error = execute(
        args(from, Some("w-41"), true),
        &adapter,
        &CommandContext::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "INVALID_ARGS");
    assert!(adapter.dragged.lock().unwrap().is_none());
}

#[test]
fn process_target_drag_refuses_headed_before_delivery() {
    let adapter = TwoInstances::default();
    let error = execute(
        args(xy(), Some("w-41"), true),
        &adapter,
        &CommandContext::default().with_headed(true),
    )
    .unwrap_err();
    assert_eq!(error.code(), "INVALID_ARGS");
    assert!(adapter.dragged.lock().unwrap().is_none());
}

#[test]
fn process_target_drag_refuses_an_unknown_window_before_delivery() {
    let adapter = TwoInstances::default();
    let error = execute(
        args(xy(), Some("w-9"), true),
        &adapter,
        &CommandContext::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "WINDOW_NOT_FOUND");
    assert!(adapter.dragged.lock().unwrap().is_none());
}

#[test]
fn window_id_without_process_target_is_refused() {
    let adapter = TwoInstances::default();
    let error = execute(
        args(xy(), Some("w-41"), false),
        &adapter,
        &CommandContext::default().with_headed(true),
    )
    .unwrap_err();
    assert_eq!(error.code(), "INVALID_ARGS");
    assert!(adapter.dragged.lock().unwrap().is_none());
}
