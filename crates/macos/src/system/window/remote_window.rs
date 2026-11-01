//! Finds an app's window that `AXWindows` does not list: a window on another
//! Space. The accessibility API only returns windows of the current Space, but
//! the app still serves its elements. A remote token (pid, a fixed tag and an
//! element id) addresses one of them directly; scanning a bounded id range
//! finds the element whose CoreGraphics window number is the one we want.
//!
//! `_AXUIElementCreateWithRemoteToken` is a private HIServices symbol, looked
//! up at run time; when it is absent the scan finds nothing and callers keep
//! their normal "not accessible" answer.

use agent_desktop_core::AdapterError;

use crate::tree::AXElement;

/// Element ids scanned per window lookup. Window elements are vended early, so
/// their ids are small; the bound keeps the scan to a few hundred
/// milliseconds even for an app that answers every id.
const MAX_ELEMENT_ID: u64 = 1_000;
/// The tag AppKit puts in every remote element token ("coco").
const TOKEN_TAG: u32 = 0x636f_636f;

type CreateWithRemoteToken =
    unsafe extern "C" fn(core_foundation_sys::data::CFDataRef) -> accessibility_sys::AXUIElementRef;

pub(crate) fn token(pid: i32, element_id: u64) -> [u8; 20] {
    let mut token = [0_u8; 20];
    token[0..4].copy_from_slice(&pid.to_le_bytes());
    token[8..12].copy_from_slice(&TOKEN_TAG.to_le_bytes());
    token[12..20].copy_from_slice(&element_id.to_le_bytes());
    token
}

/// The AX window element of `pid` whose window number is `window_number`, or
/// `None` when the scan finds no such window before the deadline or the id
/// bound. Every element inside a window reports that window's number, so a
/// match counts only when its role is `AXWindow`.
pub(crate) fn find(
    pid: i32,
    window_number: i64,
    deadline: std::time::Instant,
) -> Result<Option<AXElement>, AdapterError> {
    let Some(create) = create_fn() else {
        return Ok(None);
    };
    let started = std::time::Instant::now();
    for element_id in 0..MAX_ELEMENT_ID {
        if std::time::Instant::now() >= deadline {
            return Ok(None);
        }
        let Some(element) = element(create, pid, element_id) else {
            continue;
        };
        let Ok(Some(number)) = crate::system::window::window_bridge::window_id(&element, deadline)
        else {
            continue;
        };
        if number != window_number {
            continue;
        }
        if crate::tree::resolve::resolve_ax_read::read_string(&element, "AXRole", deadline)
            .ok()
            .flatten()
            .as_deref()
            == Some("AXWindow")
        {
            tracing::debug!(
                pid,
                window_number,
                element_id,
                elapsed_ms = started.elapsed().as_millis() as u64,
                "window: found off-Space window by remote token"
            );
            return Ok(Some(element));
        }
    }
    tracing::debug!(
        pid,
        window_number,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "window: remote token scan found no window"
    );
    Ok(None)
}

fn element(create: CreateWithRemoteToken, pid: i32, element_id: u64) -> Option<AXElement> {
    use core_foundation::base::TCFType;
    let bytes = token(pid, element_id);
    let data = core_foundation::data::CFData::from_buffer(&bytes);
    let raw = unsafe { create(data.as_concrete_TypeRef()) };
    (!raw.is_null()).then(|| AXElement(raw))
}

fn create_fn() -> Option<CreateWithRemoteToken> {
    static CREATE: std::sync::OnceLock<Option<CreateWithRemoteToken>> = std::sync::OnceLock::new();
    *CREATE.get_or_init(|| {
        let symbol = unsafe { dlsym(RTLD_DEFAULT, c"_AXUIElementCreateWithRemoteToken".as_ptr()) };
        if symbol.is_null() {
            None
        } else {
            Some(unsafe {
                std::mem::transmute::<*mut std::ffi::c_void, CreateWithRemoteToken>(symbol)
            })
        }
    })
}

const RTLD_DEFAULT: *mut std::ffi::c_void = -2_isize as *mut std::ffi::c_void;

unsafe extern "C" {
    fn dlsym(
        handle: *mut std::ffi::c_void,
        symbol: *const std::ffi::c_char,
    ) -> *mut std::ffi::c_void;
}

#[cfg(test)]
#[path = "remote_window_tests.rs"]
mod tests;
