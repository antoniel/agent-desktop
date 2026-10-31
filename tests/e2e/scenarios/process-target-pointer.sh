# A process-targeted click reaches the background fixture's button at the
# requested point while the fixture never becomes frontmost. SwiftUI buttons
# accept the first mouse in an inactive window. The drag canvas is a plain
# NSView that does not, and it sits below the fold, so surfaces.sh covers
# process-targeted drags on the focused fixture instead.

target_bounds() {
    local variable="$1" target="$2" output
    output="$(agent_exec 0 get "$(target_ref "$target")" --property bounds 2>/dev/null)" || return 1
    printf -v "$variable" '%s' "$(printf '%s' "$output" | python3 -c '
import json, sys
box = json.load(sys.stdin)["data"]["value"]
print(box["x"], box["y"], box["width"], box["height"])
')"
}

read_status click_before click-status || fail "click status is unavailable before the background click"
find_target pointer_button button primary-button || fail "primary button is not uniquely addressable for the click"
target_bounds button_box "$pointer_button" || fail "primary button bounds are unavailable"
read -r bx by bw bh <<< "$button_box"
button_point="$(python3 -c "print(f'{$bx + $bw / 2:.1f},{$by + $bh / 2:.1f}')")"
run_process_target mouse-click --xy "$button_point" --window-id "$fixture_window_id" --process-target
await_status click_after click-status different "$click_before" || fail "process-targeted click did not reach the button"
pass "process-targeted mouse-click pressed the background fixture's button"
