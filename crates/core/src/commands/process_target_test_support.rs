//! Two instances of one app, each with a window, for `--process-target`
//! pointer tests. Records which window received a background gesture and
//! fails the test if a gesture ever takes the global (cursor-moving) path.

use crate::actions::action_request::ActionRequest;
use crate::actions::action_result::ActionResult;
use crate::adapter::{ActionOps, InputOps, NativeHandle, ObservationOps, SystemOps};
use crate::{AdapterError, DragParams, MouseEvent, OperationReport};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub(crate) struct TwoInstances {
    pub(crate) clicked: Mutex<Option<(String, MouseEvent)>>,
    pub(crate) dragged: Mutex<Option<(String, DragParams)>>,
    pub(crate) typed: Mutex<Option<(String, String)>>,
    pub(crate) pressed: Mutex<Option<crate::ProcessIdentity>>,
    /// The adapter has a direct window lookup (`window_by_id`).
    pub(crate) exact: bool,
    pub(crate) inventory_reads: AtomicUsize,
}

impl TwoInstances {
    pub(crate) fn exact() -> Self {
        Self {
            exact: true,
            ..Self::default()
        }
    }

    pub(crate) fn inventory_reads(&self) -> usize {
        self.inventory_reads.load(Ordering::SeqCst)
    }
}

fn instance(pid: u32) -> crate::AppInfo {
    crate::AppInfo {
        name: "Electron".into(),
        pid: crate::ProcessId::new(pid),
        bundle_id: Some("com.github.Electron".into()),
        process_instance: Some(format!("generation-{pid}")),
        presentation: None,
    }
}

fn window(pid: u32) -> crate::WindowInfo {
    crate::WindowInfo {
        id: format!("w-{pid}"),
        title: format!("Instance {pid}"),
        app: "Electron".into(),
        pid: crate::ProcessId::new(pid),
        process_instance: Some(format!("generation-{pid}")),
        bounds: None,
        state: crate::WindowState {
            visible: Some(true),
            ..Default::default()
        },
    }
}

impl ObservationOps for TwoInstances {
    fn list_apps(&self, _deadline: crate::Deadline) -> Result<Vec<crate::AppInfo>, AdapterError> {
        self.inventory_reads.fetch_add(1, Ordering::SeqCst);
        Ok(vec![instance(41), instance(42)])
    }

    fn list_windows(
        &self,
        _filter: &crate::WindowFilter,
        _deadline: crate::Deadline,
    ) -> Result<Vec<crate::WindowInfo>, AdapterError> {
        self.inventory_reads.fetch_add(1, Ordering::SeqCst);
        Ok(vec![window(41), window(42)])
    }
}

impl ActionOps for TwoInstances {
    fn execute_action(
        &self,
        _handle: &NativeHandle,
        _request: ActionRequest,
        _lease: &crate::InteractionLease,
    ) -> Result<ActionResult, AdapterError> {
        panic!("a process-targeted gesture must never fall back to an action")
    }
}

impl InputOps for TwoInstances {
    fn mouse_event(
        &self,
        _event: MouseEvent,
        _lease: &crate::InteractionLease,
    ) -> Result<OperationReport, AdapterError> {
        panic!("a process-targeted click must never move the global cursor")
    }

    fn drag(
        &self,
        _params: DragParams,
        _lease: &crate::InteractionLease,
    ) -> Result<OperationReport, AdapterError> {
        panic!("a process-targeted drag must never move the global cursor")
    }

    fn mouse_event_for_window(
        &self,
        window: &crate::WindowInfo,
        event: MouseEvent,
        _lease: &crate::InteractionLease,
    ) -> Result<OperationReport, AdapterError> {
        let report = OperationReport::pointer_dispatched(&event, false);
        *self.clicked.lock().unwrap() = Some((window.id.clone(), event));
        Ok(report)
    }

    fn type_text_for_window(
        &self,
        window: &crate::WindowInfo,
        text: &str,
        _lease: &crate::InteractionLease,
    ) -> Result<ActionResult, AdapterError> {
        *self.typed.lock().unwrap() = Some((window.id.clone(), text.to_owned()));
        Ok(ActionResult::delivered_unverified("type_text"))
    }

    fn drag_for_window(
        &self,
        window: &crate::WindowInfo,
        params: DragParams,
        _lease: &crate::InteractionLease,
    ) -> Result<OperationReport, AdapterError> {
        let report = OperationReport::drag_dispatched(&params);
        *self.dragged.lock().unwrap() = Some((window.id.clone(), params));
        Ok(report)
    }
}

impl SystemOps for TwoInstances {
    crate::adapter::guarded_interaction_lease!();

    fn window_by_id(
        &self,
        id: &str,
        _deadline: crate::Deadline,
    ) -> Result<Option<crate::WindowInfo>, AdapterError> {
        if !self.exact {
            return Ok(None);
        }
        [41, 42]
            .into_iter()
            .map(window)
            .find(|window| window.id == id)
            .map(Some)
            .ok_or_else(|| {
                AdapterError::new(crate::ErrorCode::WindowNotFound, format!("No window {id}"))
            })
    }

    fn press_key_for_app(
        &self,
        process: crate::ProcessIdentity,
        _combo: &crate::KeyCombo,
        _policy: crate::InteractionPolicy,
        _lease: &crate::InteractionLease,
    ) -> Result<ActionResult, AdapterError> {
        *self.pressed.lock().unwrap() = Some(process);
        Ok(ActionResult::delivered_unverified("PressKey"))
    }

    fn resolve_window_strict(
        &self,
        window: &crate::WindowInfo,
        _deadline: crate::Deadline,
    ) -> Result<crate::WindowInfo, AdapterError> {
        Ok(window.clone())
    }

    fn focus_window(
        &self,
        _window: &crate::WindowInfo,
        _lease: &crate::InteractionLease,
    ) -> Result<(), AdapterError> {
        panic!("a process-targeted gesture must never focus a window")
    }
}
