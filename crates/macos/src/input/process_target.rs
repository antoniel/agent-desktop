//! Input addressed to the process that owns one window, delivered in the
//! background with `CGEventPostToPid`. The app is not activated and the
//! system cursor does not move. The process receives the events; whether the
//! app handled them is for the caller to verify, so every result is
//! `delivered_unverified`.

use agent_desktop_core::{AdapterError, Deadline, DeliverySemantics, ErrorCode, WindowInfo};

/// The process that owns the window, with its identity token.
pub(crate) struct WindowTarget {
    pub(crate) pid: i32,
    pub(crate) instance: String,
}

impl WindowTarget {
    /// Reads the window record from the window server and checks it against
    /// the window's process identity. Nothing is sent when this fails.
    pub(crate) fn resolve(window: &WindowInfo, deadline: Deadline) -> Result<Self, AdapterError> {
        let instant = crate::tree::locator_deadline::from_operation(deadline)?;
        let (_, record) =
            crate::system::window::window_resolve::locate_verified_record_until(window, instant)
                .map_err(unsent)?;
        let instance = window
            .process_instance
            .clone()
            .filter(|instance| !instance.is_empty())
            .ok_or_else(|| unsent(stale("The window has no process identity")))?;
        Ok(Self {
            pid: record.pid,
            instance,
        })
    }

    /// The process that owns the window must be the one observed. A pid that
    /// was reused by another process instance never receives the event.
    pub(crate) fn require_same_process(&self) -> Result<(), AdapterError> {
        if crate::system::process::process_identity::matches_instance(self.pid, &self.instance)? {
            return Ok(());
        }
        Err(stale("The target window's process instance changed"))
    }
}

/// Types Unicode text into the focused element of the window's process. The
/// process identity is re-checked before every chunk.
pub(crate) fn type_text(
    window: &WindowInfo,
    text: &str,
    deadline: Deadline,
) -> Result<agent_desktop_core::ActionResult, AdapterError> {
    let target = WindowTarget::resolve(window, deadline)?;
    crate::input::keyboard::synthesize_text(text, target.pid, deadline, |_| {
        target.require_same_process()
    })?;
    Ok(
        agent_desktop_core::ActionResult::delivered_unverified("type_text").with_steps(vec![
            agent_desktop_core::ActionStep::succeeded("CGEventPostToPid")
                .with_mechanism(agent_desktop_core::StepMechanism::PhysicalSynthetic)
                .with_verified(false),
        ]),
    )
}

pub(crate) fn stale(message: &str) -> AdapterError {
    AdapterError::new(ErrorCode::StaleRef, message)
        .with_suggestion("Run 'list-windows' to get the current window id, then retry.")
}

/// Marks an error raised before any event was posted as not delivered.
pub(crate) fn unsent(error: AdapterError) -> AdapterError {
    match error.disposition {
        DeliverySemantics::Unknown => error.with_disposition(DeliverySemantics::not_delivered()),
        _ => error,
    }
}
