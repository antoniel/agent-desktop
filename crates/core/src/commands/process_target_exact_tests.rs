//! With a direct window lookup, `--process-target` reads only the named
//! window: no window or application inventory.

use crate::commands::process_target_test_support::TwoInstances;
use crate::context::CommandContext;

#[test]
fn press_reaches_the_window_process_without_any_inventory() {
    let adapter = TwoInstances::exact();
    crate::commands::press::execute(
        crate::commands::press::PressArgs {
            combo: "cmd+k".into(),
            app: None,
            window_id: Some("w-42".into()),
            process_target: true,
            force: false,
        },
        &adapter,
        &CommandContext::default(),
    )
    .unwrap();
    let process = adapter.pressed.lock().unwrap().take().unwrap();
    assert_eq!(process.pid.get(), 42);
    assert_eq!(process.instance, "generation-42");
    assert_eq!(adapter.inventory_reads(), 0);
}

#[test]
fn click_drag_and_type_read_no_inventory() {
    let adapter = TwoInstances::exact();
    let context = CommandContext::default();
    crate::commands::mouse_click::execute(
        crate::commands::mouse_click::MouseClickArgs {
            x: 1.0,
            y: 2.0,
            button: crate::MouseButton::Left,
            count: 1,
            modifiers: Vec::new(),
            window_id: Some("w-41".into()),
            process_target: true,
        },
        &adapter,
        &context,
    )
    .unwrap();
    super::type_text(Some("w-41"), "hoje", &adapter, &context).unwrap();
    assert_eq!(adapter.clicked.lock().unwrap().as_ref().unwrap().0, "w-41");
    assert_eq!(adapter.typed.lock().unwrap().as_ref().unwrap().0, "w-41");
    assert_eq!(adapter.inventory_reads(), 0);
}

#[test]
fn unknown_window_is_refused_before_delivery() {
    let adapter = TwoInstances::exact();
    let error =
        super::type_text(Some("w-9"), "x", &adapter, &CommandContext::default()).unwrap_err();
    assert_eq!(error.code(), "WINDOW_NOT_FOUND");
    assert!(adapter.typed.lock().unwrap().is_none());
}
