# Process-targeted keys reach the background fixture's focused field while the
# fixture never becomes frontmost. A semantic focus first gives the field
# keyboard focus and selects its text, so the typed value replaces it.

run_process_target() {
    local action="$1" output
    shift
    safety_checkpoint || fail "ownership or frontmost identity changed before process-targeted input"
    if ! output="$(agent_exec 1 "$action" "$@")"; then
        printf '%s\n' "$output" >&2
        fail "process-targeted $action failed"
    fi
    printf '%s' "$output" | python3 "$safety_guard" process-target-action "$action" \
        >/dev/null 2>&1 || fail "process-targeted $action did not report background delivery: $output"
    sleep 0.2
    safety_checkpoint || fail "ownership or frontmost identity changed after process-targeted input"
}

find_target keys_field textfield text-input || fail "text field is not uniquely addressable for keys"
run_simple_action changed focus "$keys_field"
keys_value="safe-semantic-keys-$fixture_pid"
run_process_target type --window-id "$fixture_window_id" --process-target "$keys_value"
await_status keys_typed text-echo equal "$keys_value" || fail "process-targeted type did not reach the focused field"
pass "process-targeted type reached the background fixture's focused field without a ref"

run_process_target press delete --window-id "$fixture_window_id" --process-target
await_status keys_deleted text-echo equal "${keys_value%?}" || fail "process-targeted delete did not edit the field"
pass "process-targeted press delivered one key to the background fixture"
