//! The rights the app has to be granted by hand, and where to grant them.
//!
//! Only Android has any: the rest of the platforms either install their own
//! updates or hand that job to a store, so the list is empty there and the
//! onboarding step that reads it disappears on its own.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::error::CommandError;

/// A right the reader may have to grant. One variant per screen the app can
/// send them to — never a catalogue of everything Android knows about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionKind {
    /// "Install unknown apps", without which no update can be handed to the
    /// system package installer.
    InstallPackages,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionState {
    pub kind: PermissionKind,
    pub granted: bool,
    /// Whether the app can do anything about it, or the reader has to walk into
    /// the system settings themselves.
    pub requestable: bool,
}

/// Every right this build needs, with its current state. The interface renders
/// the list it gets; an empty one means there is nothing to ask for.
#[tauri::command]
pub fn permission_states(app: AppHandle) -> Vec<PermissionState> {
    platform::states(&app)
}

/// Opens the system screen that grants `kind`. Android answers before the
/// reader has decided anything, so the caller has to re-read the state when the
/// app comes back to the foreground.
#[tauri::command]
pub fn request_permission(app: AppHandle, kind: PermissionKind) -> Result<(), CommandError> {
    platform::request(&app, kind)
}

#[cfg(target_os = "android")]
mod platform {
    use super::*;
    use crate::android_bridge;

    pub(super) fn states(_app: &AppHandle) -> Vec<PermissionState> {
        vec![PermissionState {
            kind: PermissionKind::InstallPackages,
            // A failed lookup is reported as "not granted": claiming the right
            // is held would send the reader into an install that cannot start.
            granted: can_install_packages().unwrap_or(false),
            requestable: true,
        }]
    }

    pub(super) fn request(_app: &AppHandle, kind: PermissionKind) -> Result<(), CommandError> {
        match kind {
            PermissionKind::InstallPackages => request_install_packages(),
        }
    }

    fn can_install_packages() -> Result<bool, CommandError> {
        android_bridge::with_installer("permission_check_failed", |env, class, activity| {
            let result = env.call_static_method(
                class,
                "canInstall",
                "(Landroid/content/Context;)Z",
                &[activity.into()],
            );

            let Ok(value) = result else {
                android_bridge::clear_pending_exception(env);
                return Err(CommandError::new("permission_check_failed"));
            };

            value
                .z()
                .map_err(|_| CommandError::new("permission_check_failed"))
        })
    }

    fn request_install_packages() -> Result<(), CommandError> {
        android_bridge::with_installer("permission_request_failed", |env, class, activity| {
            let result = env.call_static_method(
                class,
                "requestInstallPermission",
                "(Landroid/content/Context;)Z",
                &[activity.into()],
            );

            let Ok(value) = result else {
                android_bridge::clear_pending_exception(env);
                return Err(CommandError::new("permission_request_failed"));
            };

            // Some vendor builds hide the screen entirely. Saying so is better
            // than leaving the reader waiting for a settings page that never
            // opens.
            match value.z() {
                Ok(true) => Ok(()),
                Ok(false) => Err(CommandError::new("permission_screen_unavailable")),
                Err(_) => Err(CommandError::new("permission_request_failed")),
            }
        })
    }
}

#[cfg(not(target_os = "android"))]
mod platform {
    use super::*;

    /// Desktop installs its own updates and iOS hands that to AltStore, so
    /// neither has a right to ask for.
    pub(super) fn states(_app: &AppHandle) -> Vec<PermissionState> {
        Vec::new()
    }

    pub(super) fn request(_app: &AppHandle, _kind: PermissionKind) -> Result<(), CommandError> {
        Err(CommandError::new("permission_screen_unavailable"))
    }
}
