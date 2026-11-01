use agent_desktop_core::{AdapterError, WindowInfo};

/// One window by id, read directly from the window server with its process
/// instance: no global window or application inventory. A window that does
/// not exist is `WINDOW_NOT_FOUND`, sent nothing.
pub(crate) fn window_by_id(
    id: &str,
    deadline: std::time::Instant,
) -> Result<WindowInfo, AdapterError> {
    let number = super::window_resolve::parse_window_number(id)
        .ok_or_else(|| super::window_resolve::invalid_window_id(id))?;
    let record =
        crate::system::window::cg_window_exact::exact_window_record_until(number, deadline)?
            .ok_or_else(|| super::window_resolve::window_not_found(id))?;
    if record.process_instance.is_none() {
        return Err(super::window_resolve::window_not_found(id));
    }
    record.into_window_info(false, None, true)
}

/// A window identity mismatch that names the field that no longer matches, so
/// a caller can tell a different window from a window whose title changed.
pub(super) fn mismatched(id: &str, field: &str) -> AdapterError {
    super::window_resolve::window_identity_mismatch(id)
        .with_details(serde_json::json!({ "kind": "window_identity_mismatch", "field": field }))
}
