use super::window_title_changed;
use crate::{AdapterError, AppError, ErrorCode};

fn mismatch(field: &str) -> AppError {
    AppError::Adapter(
        AdapterError::new(ErrorCode::WindowNotFound, "Window 'w-1' identity mismatch")
            .with_details(
                serde_json::json!({ "kind": "window_identity_mismatch", "field": field }),
            ),
    )
}

#[test]
fn only_a_title_change_is_read_again() {
    assert!(window_title_changed(&mismatch("title")));
    for field in ["pid", "process_instance", "app"] {
        assert!(!window_title_changed(&mismatch(field)));
    }
    assert!(!window_title_changed(&AppError::Adapter(
        AdapterError::new(ErrorCode::WindowNotFound, "Window 'w-1' not found",)
    )));
}
