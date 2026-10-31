//! Two instances of one app, each with a window, for `--process-target`
//! tests. Records which window's process received background input and fails
//! the test if the input ever takes a focusing or global path.

use crate::AdapterError;
use crate::actions::action_request::ActionRequest;
use crate::actions::action_result::ActionResult;
use crate::adapter::{ActionOps, InputOps, NativeHandle, ObservationOps, SystemOps};
use std::sync::Mutex;

#[derive(Default)]
pub(crate) struct TwoInstances {
    pub(crate) typed: Mutex<Option<(String, String)>>,
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
        Ok(vec![instance(41), instance(42)])
    }

    fn list_windows(
        &self,
        _filter: &crate::WindowFilter,
        _deadline: crate::Deadline,
    ) -> Result<Vec<crate::WindowInfo>, AdapterError> {
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
        panic!("process-targeted input must never fall back to an action")
    }
}

impl InputOps for TwoInstances {
    fn type_text_for_window(
        &self,
        window: &crate::WindowInfo,
        text: &str,
        _lease: &crate::InteractionLease,
    ) -> Result<ActionResult, AdapterError> {
        *self.typed.lock().unwrap() = Some((window.id.clone(), text.to_owned()));
        Ok(ActionResult::delivered_unverified("type_text"))
    }
}

impl SystemOps for TwoInstances {
    crate::adapter::guarded_interaction_lease!();

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
        panic!("process-targeted input must never focus a window")
    }
}
