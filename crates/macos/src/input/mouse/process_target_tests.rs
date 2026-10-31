use super::*;
use agent_desktop_core::DeliverySemantics;

fn target() -> WindowTarget {
    WindowTarget {
        pid: 1,
        instance: "one".into(),
        window_number: 7,
        bounds: Rect {
            x: 100.0,
            y: 50.0,
            width: 400.0,
            height: 300.0,
        },
    }
}

#[test]
fn global_point_becomes_window_local_top_left_point() {
    let local = target().local(&Point { x: 150.0, y: 80.0 }).unwrap();
    assert_eq!(local, (50.0, 30.0));
}

#[test]
fn window_edges_are_inside_the_target() {
    let target = target();
    assert_eq!(
        target.local(&Point { x: 100.0, y: 50.0 }).unwrap(),
        (0.0, 0.0)
    );
    assert_eq!(
        target.local(&Point { x: 500.0, y: 350.0 }).unwrap(),
        (400.0, 300.0)
    );
}

#[test]
fn point_outside_the_window_is_refused_without_delivery() {
    for point in [
        Point { x: 99.0, y: 80.0 },
        Point { x: 150.0, y: 351.0 },
        Point { x: 501.0, y: 80.0 },
        Point { x: 150.0, y: 49.0 },
    ] {
        let error = target().local(&point).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgs);
        assert_eq!(error.disposition, DeliverySemantics::not_delivered());
    }
}

#[test]
fn non_finite_point_is_refused_without_delivery() {
    let error = target()
        .local(&Point {
            x: f64::NAN,
            y: 80.0,
        })
        .unwrap_err();
    assert_eq!(error.disposition, DeliverySemantics::not_delivered());
}

#[test]
fn modifiers_map_to_nsevent_flags() {
    assert_eq!(modifier_flags(&[]), 0);
    assert_eq!(
        modifier_flags(&[Modifier::Meta, Modifier::Shift]),
        (1 << 20) | (1 << 17)
    );
    assert_eq!(
        modifier_flags(&[Modifier::Alt, Modifier::Ctrl]),
        (1 << 19) | (1 << 18)
    );
}

#[test]
fn buttons_map_to_bridge_codes() {
    assert_eq!(button_code(&MouseButton::Left), 0);
    assert_eq!(button_code(&MouseButton::Right), 1);
    assert_eq!(button_code(&MouseButton::Middle), 2);
}

#[test]
fn hover_and_wheel_are_refused_without_delivery() {
    let window = WindowInfo {
        id: "w-7".into(),
        title: String::new(),
        app: String::new(),
        pid: agent_desktop_core::ProcessId::new(1),
        process_instance: Some("one".into()),
        bounds: None,
        state: agent_desktop_core::WindowState::default(),
    };
    let error = click(
        &window,
        &MouseEvent {
            kind: MouseEventKind::Move,
            point: Point { x: 1.0, y: 1.0 },
            button: MouseButton::Left,
            modifiers: Vec::new(),
        },
        Deadline::standard().unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::ActionNotSupported);
    assert_eq!(error.disposition, DeliverySemantics::not_delivered());
}
