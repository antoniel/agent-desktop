use crate::{
    AppError, Modifier, MouseButton, MouseEvent, MouseEventKind, Point,
    adapter::PlatformAdapter,
    commands::{
        operation_response::operation_response,
        point_resolve::{require_cursor_policy, require_pointer_capability},
    },
    context::CommandContext,
};
use serde_json::{Value, json};

pub struct MouseClickArgs {
    pub x: f64,
    pub y: f64,
    pub button: MouseButton,
    pub count: u32,
    pub modifiers: Vec<Modifier>,
    /// With `process_target`, the window whose process receives the click.
    pub window_id: Option<String>,
    pub process_target: bool,
}

pub fn execute(
    args: MouseClickArgs,
    adapter: &dyn PlatformAdapter,
    context: &CommandContext,
) -> Result<Value, AppError> {
    if let Some(window_id) =
        super::process_target::requested(args.process_target, args.window_id.as_deref(), context)?
    {
        return background_click(window_id, &args, adapter);
    }
    if args.window_id.is_some() {
        return Err(AppError::invalid_input(
            "--window-id on mouse-click requires --process-target",
        ));
    }
    require_cursor_policy(context, "mouse-click")?;
    require_pointer_capability(adapter)?;
    crate::validate_mouse_click_count(args.count)?;
    let _cursor = crate::cursor_overlay::hold_cursor(context, crate::Deadline::standard()?);
    let lease = crate::commands::helpers::acquire_interaction_lease(adapter)?;
    let point = Point {
        x: args.x,
        y: args.y,
    };
    point.validate()?;
    let result = crate::cursor_overlay::dispatch_mouse_event_with_cursor(
        adapter,
        context,
        MouseEvent {
            kind: MouseEventKind::Click { count: args.count },
            point,
            button: args.button,
            modifiers: args.modifiers,
        },
        true,
        &lease,
    );
    operation_response(
        &result?,
        json!({ "x": args.x, "y": args.y, "count": args.count }),
    )
}

/// Delivers the click to the named window's process in the background: the
/// cursor does not move and the app is not activated.
fn background_click(
    window_id: &str,
    args: &MouseClickArgs,
    adapter: &dyn PlatformAdapter,
) -> Result<Value, AppError> {
    crate::validate_mouse_click_count(args.count)?;
    let point = Point {
        x: args.x,
        y: args.y,
    };
    point.validate()?;
    let deadline = crate::Deadline::standard()?;
    let (window, lease) = super::process_target::resolve_window(window_id, adapter, deadline)?;
    let report = adapter.mouse_event_for_window(
        &window,
        MouseEvent {
            kind: MouseEventKind::Click { count: args.count },
            point,
            button: args.button.clone(),
            modifiers: args.modifiers.clone(),
        },
        &lease,
    )?;
    operation_response(
        &report,
        json!({
            "x": args.x,
            "y": args.y,
            "count": args.count,
            "window_id": window.id,
            "process_target": true,
        }),
    )
}

#[cfg(test)]
#[path = "mouse_click_process_target_tests.rs"]
mod process_target_tests;

#[cfg(test)]
#[path = "mouse_click_tests.rs"]
mod tests;
