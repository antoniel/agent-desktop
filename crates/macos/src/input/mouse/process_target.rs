//! Pointer input addressed to one window of a background process.
//!
//! The default pointer path posts at the HID tap: the system cursor moves and
//! the window under it receives the event, so it needs `--headed`. This path
//! posts each event to the owning process with `CGEventPostToPid`, bound to the
//! named window. The system cursor does not move and no application is
//! activated. The process receives the event; whether the app handled it is
//! for the caller to verify, so every report is `delivered_unverified`.

use crate::input::process_target::{WindowTarget, unsent};
use agent_desktop_core::{
    AdapterError, Deadline, DragParams, ErrorCode, Modifier, MouseButton, MouseEvent,
    MouseEventKind, Point, Rect, WindowInfo,
};
use std::time::Duration;

const DOWN: i32 = 0;
const UP: i32 = 1;
const DRAGGED: i32 = 2;
const CLICK_HOLD: Duration = Duration::from_millis(30);
const CLICK_GAP: Duration = Duration::from_millis(30);
const PICKUP_DELAY: Duration = Duration::from_millis(100);
const DEFAULT_DURATION_MS: u64 = 300;
const DEFAULT_DROP_DELAY_MS: u64 = 100;
const DWELL_TICK_MS: u64 = 16;
const MAX_STEPS: u64 = 4_096;

impl WindowTarget {
    /// Converts a global point to the window-local, top-left point the
    /// window receives. A point outside the window frame is refused: the
    /// event would reach the process, but no view of this window.
    fn local(&self, point: &Point) -> Result<(f64, f64), AdapterError> {
        crate::input::mouse::validate_point(point).map_err(unsent)?;
        let Rect {
            x,
            y,
            width,
            height,
        } = self.bounds;
        if point.x < x || point.y < y || point.x > x + width || point.y > y + height {
            return Err(unsent(
                AdapterError::new(
                    ErrorCode::InvalidArgs,
                    "The point is outside the target window",
                )
                .with_details(serde_json::json!({
                    "point": { "x": point.x, "y": point.y },
                    "window_bounds": { "x": x, "y": y, "width": width, "height": height },
                }))
                .with_suggestion(
                    "Use global screen coordinates inside the window frame from 'list-windows'.",
                ),
            ));
        }
        Ok((point.x - x, point.y - y))
    }

    fn post(
        &self,
        kind: i32,
        button: i32,
        local: (f64, f64),
        click_count: i64,
        flags: u64,
    ) -> Result<(), AdapterError> {
        let status = unsafe {
            agent_desktop_post_window_mouse(
                self.pid,
                self.window_number,
                kind,
                button,
                local.0,
                local.1,
                click_count,
                flags,
            )
        };
        match status {
            0 => Ok(()),
            1 => Err(AdapterError::new(
                ErrorCode::InvalidArgs,
                "Invalid window mouse event arguments",
            )),
            _ => Err(AdapterError::internal(
                "Could not create a window-bound mouse event",
            )),
        }
    }
}

/// Clicks `count` times at a global point inside the window. A delivered press
/// is always followed by its release, even past the deadline: a held button
/// would leave the app mid-gesture.
pub(crate) fn click(
    window: &WindowInfo,
    event: &MouseEvent,
    deadline: Deadline,
) -> Result<(), AdapterError> {
    let MouseEventKind::Click { count } = event.kind else {
        return Err(unsent(AdapterError::new(
            ErrorCode::ActionNotSupported,
            "Process-targeted pointer input supports clicks and drags only",
        )));
    };
    agent_desktop_core::validate_mouse_click_count(count).map_err(unsent)?;
    let target = WindowTarget::resolve(window, deadline)?;
    let local = target.local(&event.point)?;
    let button = button_code(&event.button);
    let flags = modifier_flags(&event.modifiers);
    let mut delivery = crate::actions::DeliveryTracker::default();
    for index in 1..=i64::from(count) {
        crate::input::mouse::ensure_budget(deadline, delivery)?;
        target
            .require_same_process()
            .map_err(|error| delivery.annotate(error))?;
        target
            .post(DOWN, button, local, index, flags)
            .map_err(|error| delivery.annotate(error))?;
        delivery.mark_delivered();
        std::thread::sleep(CLICK_HOLD);
        target
            .post(UP, button, local, index, flags)
            .map_err(|error| delivery.annotate(error))?;
        delivery.mark_delivered();
        if index < i64::from(count) {
            crate::input::mouse::sleep_bounded(deadline, CLICK_GAP, delivery)?;
        }
    }
    Ok(())
}

/// Drags with the left button between two global points inside the window.
/// The button is released where the pointer last was, on success or failure,
/// so the app never keeps a pressed button.
pub(crate) fn drag(
    window: &WindowInfo,
    params: &DragParams,
    deadline: Deadline,
) -> Result<(), AdapterError> {
    params.validate(deadline).map_err(unsent)?;
    let target = WindowTarget::resolve(window, deadline)?;
    let from = target.local(&params.from)?;
    let to = target.local(&params.to)?;
    let duration_ms = params.duration_ms.unwrap_or(DEFAULT_DURATION_MS);
    let steps = duration_ms.div_ceil(DWELL_TICK_MS).clamp(1, MAX_STEPS);
    let step_delay = Duration::from_secs_f64(duration_ms as f64 / steps as f64 / 1_000.0);
    let drop_delay = Duration::from_millis(params.drop_delay_ms.unwrap_or(DEFAULT_DROP_DELAY_MS));
    let mut delivery = crate::actions::DeliveryTracker::default();
    crate::input::mouse::ensure_budget(deadline, delivery)?;
    target.require_same_process().map_err(unsent)?;
    target.post(DOWN, 0, from, 1, 0).map_err(unsent)?;
    delivery.mark_delivered();
    let mut last = from;
    let outcome = (|| {
        crate::input::mouse::sleep_bounded(deadline, PICKUP_DELAY, delivery)?;
        for index in 1..=steps {
            let t = index as f64 / steps as f64;
            let point = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            target.require_same_process()?;
            target.post(DRAGGED, 0, point, 1, 0)?;
            delivery.mark_delivered();
            last = point;
            crate::input::mouse::sleep_bounded(deadline, step_delay, delivery)?;
        }
        crate::input::mouse::sleep_bounded(deadline, drop_delay, delivery)?;
        Ok(())
    })();
    let release = target.post(UP, 0, last, 1, 0);
    delivery.mark_delivered();
    outcome.map_err(|error: AdapterError| delivery.annotate(error))?;
    release.map_err(|error| delivery.annotate(error))
}

fn button_code(button: &MouseButton) -> i32 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
    }
}

/// NSEventModifierFlags bits.
fn modifier_flags(modifiers: &[Modifier]) -> u64 {
    modifiers.iter().fold(0, |flags, modifier| {
        flags
            | match modifier {
                Modifier::Shift => 1 << 17,
                Modifier::Ctrl => 1 << 18,
                Modifier::Alt => 1 << 19,
                Modifier::Meta => 1 << 20,
            }
    })
}

unsafe extern "C" {
    #[allow(clippy::too_many_arguments)]
    fn agent_desktop_post_window_mouse(
        pid: i32,
        window_number: i64,
        kind: i32,
        button: i32,
        local_x: f64,
        local_y_from_top: f64,
        click_count: i64,
        modifier_flags: u64,
    ) -> i32;
}

#[cfg(test)]
#[path = "process_target_tests.rs"]
mod tests;
