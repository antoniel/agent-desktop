use super::*;

#[test]
fn token_layout_is_pid_zero_tag_and_element_id() {
    let token = token(0x0102_0304, 0x1122_3344_5566_7788);
    assert_eq!(&token[0..4], &[0x04, 0x03, 0x02, 0x01]);
    assert_eq!(&token[4..8], &[0, 0, 0, 0]);
    assert_eq!(&token[8..12], b"ococ");
    assert_eq!(
        &token[12..20],
        &[0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11]
    );
}

#[test]
fn scan_for_a_missing_window_ends_without_a_match() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    let found = find(std::process::id() as i32, i64::from(u32::MAX), deadline).unwrap();
    assert!(found.is_none());
}
