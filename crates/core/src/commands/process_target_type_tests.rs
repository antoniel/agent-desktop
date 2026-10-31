use super::type_text;
use crate::commands::process_target_test_support::TwoInstances;
use crate::context::CommandContext;

#[test]
fn background_text_reaches_only_the_named_window_process() {
    let adapter = TwoInstances::default();
    let response = type_text(
        Some("w-41"),
        "Sintético **hoje**",
        &adapter,
        &CommandContext::default(),
    )
    .unwrap();
    let (window, text) = adapter.typed.lock().unwrap().take().unwrap();
    assert_eq!(window, "w-41");
    assert_eq!(text, "Sintético **hoje**");
    assert_eq!(response["window_id"], "w-41");
    assert_eq!(response["process_target"], true);
}

#[test]
fn background_text_requires_a_window() {
    let adapter = TwoInstances::default();
    let error = type_text(None, "x", &adapter, &CommandContext::default()).unwrap_err();
    assert_eq!(error.code(), "INVALID_ARGS");
    assert!(adapter.typed.lock().unwrap().is_none());
}

#[test]
fn background_text_refuses_headed_and_empty_text_before_delivery() {
    let adapter = TwoInstances::default();
    let headed = CommandContext::default().with_headed(true);
    assert_eq!(
        type_text(Some("w-41"), "x", &adapter, &headed)
            .unwrap_err()
            .code(),
        "INVALID_ARGS"
    );
    assert_eq!(
        type_text(Some("w-41"), "", &adapter, &CommandContext::default())
            .unwrap_err()
            .code(),
        "INVALID_ARGS"
    );
    assert!(adapter.typed.lock().unwrap().is_none());
}

#[test]
fn background_text_refuses_an_unknown_window_before_delivery() {
    let adapter = TwoInstances::default();
    let error = type_text(Some("w-9"), "x", &adapter, &CommandContext::default()).unwrap_err();
    assert_eq!(error.code(), "WINDOW_NOT_FOUND");
    assert!(adapter.typed.lock().unwrap().is_none());
}
