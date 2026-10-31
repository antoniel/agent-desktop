//! Shared contract for `--process-target`: input addressed to the process that
//! owns a named window, delivered in the background without activating it or
//! moving the cursor. The caller names the window with `--window-id`; the
//! adapter re-verifies the window and its process instance before each event.

use crate::{AppError, InteractionLease, WindowInfo, adapter::PlatformAdapter};

/// Refuses a malformed `--process-target` request before anything resolves.
/// Returns the window id when process targeting is requested.
pub(crate) fn requested<'a>(
    process_target: bool,
    window_id: Option<&'a str>,
    context: &crate::context::CommandContext,
) -> Result<Option<&'a str>, AppError> {
    if !process_target {
        return Ok(None);
    }
    let Some(window_id) = window_id else {
        return Err(AppError::invalid_input(
            "--process-target requires --window-id",
        ));
    };
    if context.physical_input_policy().is_headed() {
        return Err(AppError::invalid_input(
            "--process-target cannot be used with --headed",
        ));
    }
    Ok(Some(window_id))
}

/// Resolves the named window, then re-checks its owning app under the
/// interaction lease so a process that exited in between gets no input.
pub(crate) fn resolve_window(
    window_id: &str,
    adapter: &dyn PlatformAdapter,
    deadline: crate::Deadline,
) -> Result<(WindowInfo, InteractionLease), AppError> {
    let window = crate::snapshot::resolve_window(adapter, None, Some(window_id), deadline)?;
    let app = crate::system::app_lookup::resolve_app_owning(&window, adapter, deadline)?;
    let lease = adapter.acquire_interaction_lease(deadline)?;
    crate::system::app_lookup::revalidate_app_for_mutation(adapter, &app, lease.deadline())?;
    Ok((window, lease))
}

/// Types `text` into the focused element of the named window's process, in
/// the background: no REF, no activation, no cursor. The process receives the
/// Unicode key events; whether the app inserted them is for the caller to
/// verify.
pub fn type_text(
    window_id: Option<&str>,
    text: &str,
    adapter: &dyn PlatformAdapter,
    context: &crate::context::CommandContext,
) -> Result<serde_json::Value, AppError> {
    let window_id = requested(true, window_id, context)?.unwrap_or_default();
    if text.is_empty() {
        return Err(AppError::invalid_input("type --process-target needs TEXT"));
    }
    if text.len() > MAX_TEXT_LEN {
        return Err(AppError::invalid_input(format!(
            "Text exceeds maximum length of {MAX_TEXT_LEN} characters"
        )));
    }
    let budget = super::type_text::default_timeout_ms(text);
    let deadline = crate::Deadline::after(budget)?;
    let (window, lease) = resolve_window(window_id, adapter, deadline)?;
    let result = adapter.type_text_for_window(&window, text, &lease)?;
    let mut value = serde_json::to_value(result)?;
    mark(&mut value, Some(&window.id));
    drop(lease);
    super::helpers::apply_scoped_post_action_wait(value, None, Some(window.id), adapter, context)
}

/// Adds `window_id` and `process_target: true` to a process-targeted result,
/// so a caller can tell it from a focused-window delivery.
pub(crate) fn mark(value: &mut serde_json::Value, window_id: Option<&str>) {
    if let Some(object) = value.as_object_mut() {
        object.insert("window_id".into(), serde_json::json!(window_id));
        object.insert("process_target".into(), serde_json::json!(true));
    }
}

const MAX_TEXT_LEN: usize = 10_000;

#[cfg(test)]
#[path = "process_target_type_tests.rs"]
mod type_tests;
