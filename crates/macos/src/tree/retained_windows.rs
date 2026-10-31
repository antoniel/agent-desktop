//! Window handles kept for the life of a command host session.
//!
//! `AXWindows` lists only the windows on the current Space. A window that a
//! session observed keeps its accessibility handle here, keyed by window id,
//! pid and process instance, so the session can still read and act on it
//! after the window moves to another Space. A window the session never saw is
//! looked up by window number instead (`remote_window`).

use std::cell::RefCell;
use std::collections::VecDeque;

use crate::tree::AXElement;
use agent_desktop_core::WindowInfo;

/// Handles kept per session; the oldest is dropped first.
const MAX_WINDOWS: usize = 64;

type WindowKey = (String, u32, String);
thread_local! {
    static WINDOWS: RefCell<VecDeque<(WindowKey, AXElement)>> = const { RefCell::new(VecDeque::new()) };
}

fn key(window: &WindowInfo) -> Option<WindowKey> {
    Some((
        window.id.clone(),
        window.pid.get(),
        window.process_instance.clone()?,
    ))
}

/// Keeps the handle of a window that resolved, while a session is open.
pub(crate) fn save(window: &WindowInfo, element: &AXElement) {
    if !crate::tree::resolve::retained::active() {
        return;
    }
    let Some(key) = key(window) else {
        return;
    };
    WINDOWS.with(|slot| {
        let mut windows = slot.borrow_mut();
        windows.retain(|(old, _)| *old != key);
        if windows.len() >= MAX_WINDOWS {
            windows.pop_front();
        }
        windows.push_back((key, element.clone()));
    });
}

/// The kept handle for exactly this window id, pid and process instance.
pub(crate) fn get(id: &str, pid: u32, instance: &str) -> Option<AXElement> {
    if !crate::tree::resolve::retained::active() {
        return None;
    }
    WINDOWS.with(|slot| {
        slot.borrow()
            .iter()
            .find(|(key, _)| key.0 == id && key.1 == pid && key.2 == instance)
            .map(|(_, value)| value.clone())
    })
}

/// Drops every kept handle; runs when the session ends.
pub(crate) fn clear() {
    WINDOWS.with(|slot| slot.borrow_mut().clear());
}

/// The window element to use for `window`: the one `AXWindows` returned,
/// kept for later, or else a kept handle that still names this window, or
/// else a window found by number on another Space.
pub(crate) fn resolve(
    window: &WindowInfo,
    current: Option<AXElement>,
    deadline: std::time::Instant,
) -> Result<Option<AXElement>, agent_desktop_core::AdapterError> {
    if let Some(element) = current {
        save(window, &element);
        return Ok(Some(element));
    }
    let Some(instance) = window.process_instance.as_deref() else {
        return Ok(None);
    };
    if let Some(element) = checked(&window.id, window.pid.get(), instance, deadline)? {
        return Ok(Some(element));
    }
    discover(window, deadline)
}

/// A window `AXWindows` does not list (it is on another Space) and that this
/// session never observed: look it up by remote token. The window record was
/// already verified against CoreGraphics by the caller, so the window number
/// and pid are the live ones.
#[cfg(target_os = "macos")]
fn discover(
    window: &WindowInfo,
    deadline: std::time::Instant,
) -> Result<Option<AXElement>, agent_desktop_core::AdapterError> {
    let Some(number) = crate::system::window::window_resolve::parse_window_number(&window.id)
    else {
        return Ok(None);
    };
    let pid = crate::system::process::process_identity::to_pid_t(window.pid)?;
    let found = crate::system::window::remote_window::find(pid, number, deadline)?;
    if let Some(element) = &found {
        save(window, element);
    }
    Ok(found)
}

#[cfg(not(target_os = "macos"))]
fn discover(
    _window: &WindowInfo,
    _deadline: std::time::Instant,
) -> Result<Option<AXElement>, agent_desktop_core::AdapterError> {
    Ok(None)
}

/// A kept handle for the window a ref was observed in, when it still names
/// that window.
pub(crate) fn for_entry(
    entry: &agent_desktop_core::RefEntry,
    deadline: std::time::Instant,
) -> Result<Option<AXElement>, agent_desktop_core::AdapterError> {
    let (Some(id), Some(instance)) = (
        entry.source.source_window_id.as_deref(),
        entry.process.process_instance.as_deref(),
    ) else {
        return Ok(None);
    };
    checked(id, entry.process.pid.get(), instance, deadline)
}

/// A retained handle is reused only while it still names the same window of
/// the same process instance. A process that exited (its pid may now belong to
/// another instance), an element that no longer answers, or one that now
/// reports another window number is dropped, never returned.
fn checked(
    id: &str,
    pid: u32,
    instance: &str,
    deadline: std::time::Instant,
) -> Result<Option<AXElement>, agent_desktop_core::AdapterError> {
    let Some(element) = get(id, pid, instance) else {
        return Ok(None);
    };
    let Some(expected) = crate::system::window::window_resolve::parse_window_number(id) else {
        evict(id, pid, instance);
        return Ok(None);
    };
    let same_process = i32::try_from(pid).ok().is_some_and(|pid| {
        crate::system::process::process_identity::matches_instance(pid, instance).unwrap_or(false)
    });
    if !same_process {
        evict(id, pid, instance);
        return Ok(None);
    }
    match crate::system::window::window_resolve::ax_window_id_with_deadline(&element, deadline) {
        Ok(Some(actual)) if actual == expected => Ok(Some(element)),
        Err(error) if std::time::Instant::now() >= deadline => Err(error),
        _ => {
            evict(id, pid, instance);
            Ok(None)
        }
    }
}

fn evict(id: &str, pid: u32, instance: &str) {
    WINDOWS.with(|slot| {
        slot.borrow_mut()
            .retain(|(key, _)| !(key.0 == id && key.1 == pid && key.2 == instance));
    });
}

#[cfg(all(test, target_os = "macos"))]
#[path = "retained_windows_tests.rs"]
mod tests;
