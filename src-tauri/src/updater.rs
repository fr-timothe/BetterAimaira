//! Update delivery.
//!
//! One command surface, three mechanisms, because the platforms do not offer
//! the same thing:
//!
//! - Desktop (Windows, macOS, Linux): the signed Tauri updater downloads and
//!   installs the new bundle in place, then the app restarts itself.
//! - Android: no in-place update exists for a Tauri APK, so the new APK is
//!   downloaded to the app cache and handed to the system package installer.
//! - iOS: sideloaded builds are installed by AltStore/SideStore, which owns
//!   both refresh and update. The app can only report that a newer build is
//!   published in the source, then deep-link into the store.
//!
//! Every platform reads the same feed, published on the `gh-pages` branch and
//! served by GitHub Pages: `latest.json` for desktop and Android,
//! `altstore.json` for iOS. `scripts/release-manifest.mjs` writes both and the
//! release workflow commits them under the channel the tag belongs to.
//!
//! The feed is *not* read from `releases/latest/download`: that path only ever
//! resolves to the newest release that is not flagged as a prerelease, so a
//! project shipping betas gets a 404 and every platform reports a failed check.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::error::CommandError;

/// GitHub Pages, `gh-pages` branch: one directory per channel, each holding the
/// two manifests. A static host keeps the URL independent of how GitHub labels
/// a release, which is what broke the previous `releases/latest` feed.
const FEED_BASE: &str = "https://fr-timothe.github.io/BetterAimaira/updates";

#[cfg(any(target_os = "android", target_os = "ios"))]
const MANIFEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

/// Which release stream a build follows. `stable` only ever carries releases
/// tagged without a prerelease suffix; `beta` carries everything, because a
/// finished stable release supersedes the betas that led to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateChannel {
    Stable,
    Beta,
}

impl UpdateChannel {
    fn slug(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
        }
    }

    /// The channel a build follows until the user picks another one.
    ///
    /// A version carrying a prerelease suffix can only have come from the beta
    /// stream, and pointing such a build at `stable` would strand it: the
    /// newest stable is older than the beta already installed, so the app would
    /// report itself up to date forever.
    fn of_build(version: &semver::Version) -> Self {
        if version.pre.is_empty() {
            Self::Stable
        } else {
            Self::Beta
        }
    }
}

#[cfg(any(desktop, target_os = "android"))]
fn manifest_url(channel: UpdateChannel) -> String {
    format!("{FEED_BASE}/{}/latest.json", channel.slug())
}

#[cfg(target_os = "ios")]
fn altstore_source_url(channel: UpdateChannel) -> String {
    format!("{FEED_BASE}/{}/altstore.json", channel.slug())
}

/// How the pending update reaches the device. The interface needs this to label
/// its own button honestly: installing in place, opening the Android installer,
/// or bouncing to AltStore.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateDelivery {
    /// Downloaded, signature-checked and installed by the app itself.
    #[cfg(desktop)]
    InApp,
    /// Downloaded by the app, installed by the Android package installer.
    #[cfg(target_os = "android")]
    AndroidPackage,
    /// Installed by AltStore or SideStore from the published source.
    #[cfg(target_os = "ios")]
    AltStore,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub notes: Option<String>,
    pub published_at: Option<String>,
    pub delivery: UpdateDelivery,
    /// The channel this answer was read from, so the interface never labels a
    /// result with a channel the user changed in the meantime.
    pub channel: UpdateChannel,
    /// Direct asset link, for the platforms where the user may need it.
    pub download_url: Option<String>,
    /// Deep link that starts the install on iOS, `null` everywhere else.
    pub store_url: Option<String>,
}

impl UpdateInfo {
    fn up_to_date(current: String, delivery: UpdateDelivery, channel: UpdateChannel) -> Self {
        Self {
            available: false,
            current_version: current,
            latest_version: None,
            notes: None,
            published_at: None,
            delivery,
            channel,
            download_url: None,
            store_url: None,
        }
    }
}

/// What the app did with the request. Desktop never returns: it restarts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOutcome {
    /// The install continues outside the app (Android installer, AltStore).
    pub handed_off: bool,
    /// Android only: the user was sent to the "install unknown apps" screen and
    /// has to come back and retry once the right is granted.
    pub permission_required: bool,
}

/// Download progress, emitted while the payload streams in.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgress {
    downloaded: u64,
    total: Option<u64>,
}

const PROGRESS_EVENT: &str = "update://download-progress";
const DOWNLOADED_EVENT: &str = "update://downloaded";

/// Android only: the package installer answers on a broadcast, long after the
/// command returned, so its verdict arrives as an event instead of a result.
#[cfg(target_os = "android")]
const INSTALL_STATUS_EVENT: &str = "update://install-status";

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn check_for_update(
    app: AppHandle,
    channel: Option<UpdateChannel>,
) -> Result<UpdateInfo, CommandError> {
    let channel = channel.unwrap_or_else(|| UpdateChannel::of_build(&app.package_info().version));
    platform::check(&app, channel).await
}

#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    channel: Option<UpdateChannel>,
) -> Result<InstallOutcome, CommandError> {
    let channel = channel.unwrap_or_else(|| UpdateChannel::of_build(&app.package_info().version));
    platform::install(&app, channel).await
}

/// The channel this build belongs to. The selector reads it once so a fresh
/// install lands on the right stream without the user touching a setting.
#[tauri::command]
pub fn default_update_channel(app: AppHandle) -> UpdateChannel {
    UpdateChannel::of_build(&app.package_info().version)
}

// ---------------------------------------------------------------------------
// Android: package installer verdict, reported back from Kotlin
// ---------------------------------------------------------------------------

/// Publishes the handle the JNI callback below needs.
///
/// The package installer replies on a broadcast that outlives the command which
/// started the install, so the handle used to forward that reply cannot be the
/// one borrowed by the command.
#[cfg(target_os = "android")]
pub fn remember_app_handle(app: &AppHandle) {
    let _ = android_install_status::APP.set(app.clone());
}

#[cfg(target_os = "android")]
mod android_install_status {
    use super::{AppHandle, Serialize, INSTALL_STATUS_EVENT};
    use std::sync::OnceLock;
    use tauri::Emitter;

    pub(super) static APP: OnceLock<AppHandle> = OnceLock::new();

    #[derive(Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct InstallStatus {
        succeeded: bool,
        /// The system's own wording when it has one, so a refused install can
        /// say why instead of just failing.
        message: Option<String>,
    }

    /// Called by `ApkInstaller.kt` once `PackageInstaller` reaches a terminal
    /// status. Without it a failed or cancelled install left the interface
    /// claiming the system installer had taken over.
    ///
    /// The name, the class and the signature must stay in sync with the
    /// `external fun` declared in `ApkInstaller.kt`.
    #[allow(non_snake_case)]
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_betteraimaira_app_ApkInstaller_reportInstallStatus(
        mut env: jni::JNIEnv,
        _this: jni::objects::JObject,
        succeeded: jni::sys::jboolean,
        message: jni::objects::JString,
    ) {
        let Some(app) = APP.get() else {
            return;
        };

        // Kotlin always passes a string, empty when the system said nothing, so
        // there is no null to guard against here.
        let message: String = match env.get_string(&message) {
            Ok(value) => value.into(),
            Err(_) => {
                let _ = env.exception_clear();
                String::new()
            }
        };

        let _ = app.emit(
            INSTALL_STATUS_EVENT,
            InstallStatus {
                succeeded: succeeded != 0,
                message: (!message.is_empty()).then_some(message),
            },
        );
    }
}

// ---------------------------------------------------------------------------
// Desktop: signed in-place install through the updater plugin
// ---------------------------------------------------------------------------

#[cfg(desktop)]
mod platform {
    use super::*;
    use tauri::Emitter;
    use tauri_plugin_updater::{Updater, UpdaterExt};

    /// The endpoint is set here rather than read from `tauri.conf.json`, because
    /// the channel is a runtime choice and the configured endpoint is baked in
    /// at build time.
    fn updater(app: &AppHandle, channel: UpdateChannel) -> Result<Updater, CommandError> {
        let endpoint = manifest_url(channel)
            .parse()
            .map_err(|_| CommandError::new("update_check_failed"))?;

        app.updater_builder()
            .endpoints(vec![endpoint])
            .map_err(|_| CommandError::new("update_check_failed"))?
            .build()
            .map_err(|_| CommandError::new("update_check_failed"))
    }

    pub(super) async fn check(
        app: &AppHandle,
        channel: UpdateChannel,
    ) -> Result<UpdateInfo, CommandError> {
        let current = app.package_info().version.to_string();
        let update = updater(app, channel)?
            .check()
            .await
            .map_err(|_| CommandError::new("update_check_failed"))?;

        let Some(update) = update else {
            return Ok(UpdateInfo::up_to_date(
                current,
                UpdateDelivery::InApp,
                channel,
            ));
        };

        Ok(UpdateInfo {
            available: true,
            current_version: current,
            latest_version: Some(update.version.clone()),
            notes: update.body.clone(),
            published_at: update.date.map(|date| date.to_string()),
            delivery: UpdateDelivery::InApp,
            channel,
            download_url: None,
            store_url: None,
        })
    }

    pub(super) async fn install(
        app: &AppHandle,
        channel: UpdateChannel,
    ) -> Result<InstallOutcome, CommandError> {
        // The `Update` handle from a previous check is not kept: re-resolving it
        // costs one small JSON request and removes a stale-handle failure mode.
        let update = updater(app, channel)?
            .check()
            .await
            .map_err(|_| CommandError::new("update_check_failed"))?
            .ok_or_else(|| CommandError::new("update_not_available"))?;

        let mut downloaded: u64 = 0;
        let progress_app = app.clone();
        let finished_app = app.clone();

        update
            .download_and_install(
                move |chunk, total| {
                    downloaded += chunk as u64;
                    let _ =
                        progress_app.emit(PROGRESS_EVENT, DownloadProgress { downloaded, total });
                },
                move || {
                    let _ = finished_app.emit(DOWNLOADED_EVENT, ());
                },
            )
            .await
            .map_err(|_| CommandError::new("update_install_failed"))?;

        // On Windows the NSIS installer takes over and terminates this process;
        // elsewhere the new bundle is already in place. Restarting covers both.
        app.restart();
    }
}

// ---------------------------------------------------------------------------
// Android: APK download + system package installer
// ---------------------------------------------------------------------------

#[cfg(target_os = "android")]
mod platform {
    use super::*;
    use futures_util::StreamExt;
    use std::io::Write;
    use tauri::{Emitter, Manager};

    /// Tauri's `latest.json`. Desktop entries are keyed by target triple; the
    /// Android APK is published under `android-universal`.
    #[derive(Deserialize)]
    struct ReleaseManifest {
        version: String,
        notes: Option<String>,
        pub_date: Option<String>,
        platforms: std::collections::HashMap<String, ManifestPlatform>,
    }

    #[derive(Deserialize)]
    struct ManifestPlatform {
        url: String,
    }

    const ANDROID_KEY: &str = "android-universal";

    async fn fetch_manifest(channel: UpdateChannel) -> Result<ReleaseManifest, CommandError> {
        let client = reqwest::Client::builder()
            .timeout(MANIFEST_TIMEOUT)
            .build()
            .map_err(|_| CommandError::new("update_check_failed"))?;

        let response = client
            .get(manifest_url(channel))
            .send()
            .await
            .map_err(|_| CommandError::new("update_check_failed"))?;

        if !response.status().is_success() {
            return Err(CommandError::new("update_check_failed"));
        }

        let body = response
            .text()
            .await
            .map_err(|_| CommandError::new("update_check_failed"))?;

        serde_json::from_str::<ReleaseManifest>(&body)
            .map_err(|_| CommandError::new("update_manifest_invalid"))
    }

    fn newer_than(candidate: &str, current: &semver::Version) -> bool {
        semver::Version::parse(candidate.trim_start_matches('v'))
            .map(|remote| remote > *current)
            .unwrap_or(false)
    }

    pub(super) async fn check(
        app: &AppHandle,
        channel: UpdateChannel,
    ) -> Result<UpdateInfo, CommandError> {
        let current = app.package_info().version.clone();
        let manifest = fetch_manifest(channel).await?;
        let apk = manifest.platforms.get(ANDROID_KEY);

        if !newer_than(&manifest.version, &current) || apk.is_none() {
            return Ok(UpdateInfo::up_to_date(
                current.to_string(),
                UpdateDelivery::AndroidPackage,
                channel,
            ));
        }

        Ok(UpdateInfo {
            available: true,
            current_version: current.to_string(),
            latest_version: Some(manifest.version),
            notes: manifest.notes,
            published_at: manifest.pub_date,
            delivery: UpdateDelivery::AndroidPackage,
            channel,
            download_url: apk.map(|entry| entry.url.clone()),
            store_url: None,
        })
    }

    pub(super) async fn install(
        app: &AppHandle,
        channel: UpdateChannel,
    ) -> Result<InstallOutcome, CommandError> {
        let current = app.package_info().version.clone();
        let manifest = fetch_manifest(channel).await?;

        if !newer_than(&manifest.version, &current) {
            return Err(CommandError::new("update_not_available"));
        }

        let url = manifest
            .platforms
            .get(ANDROID_KEY)
            .map(|entry| entry.url.clone())
            .ok_or_else(|| CommandError::new("update_manifest_invalid"))?;

        let apk_path = download_apk(app, &url, &manifest.version).await?;

        // Handing the APK over copies it into the installer session, so it is a
        // long blocking call: on the async runtime it would stall every other
        // command for the length of the copy.
        let status = tauri::async_runtime::spawn_blocking(move || install_apk(&apk_path))
            .await
            .map_err(|_| CommandError::new("update_install_failed"))??;

        match status.as_str() {
            "installing" => Ok(InstallOutcome {
                handed_off: true,
                permission_required: false,
            }),
            "permission_required" => Ok(InstallOutcome {
                handed_off: false,
                permission_required: true,
            }),
            _ => Err(CommandError::new("update_install_failed")),
        }
    }

    /// Streams the APK into the app cache. A partial file from an interrupted
    /// run is never handed to the installer: the download writes to `.part`
    /// and is renamed only once the body ends.
    async fn download_apk(
        app: &AppHandle,
        url: &str,
        version: &str,
    ) -> Result<std::path::PathBuf, CommandError> {
        let directory = app
            .path()
            .app_cache_dir()
            .map_err(|_| CommandError::new("update_download_failed"))?
            .join("updates");
        std::fs::create_dir_all(&directory)
            .map_err(|_| CommandError::new("update_download_failed"))?;

        let target = directory.join(format!("betteraimaira-{version}.apk"));
        let partial = directory.join(format!("betteraimaira-{version}.apk.part"));

        let response = reqwest::get(url)
            .await
            .map_err(|_| CommandError::new("update_download_failed"))?;
        if !response.status().is_success() {
            return Err(CommandError::new("update_download_failed"));
        }

        let total = response.content_length();
        let mut downloaded: u64 = 0;
        let mut file = std::fs::File::create(&partial)
            .map_err(|_| CommandError::new("update_download_failed"))?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| CommandError::new("update_download_failed"))?;
            file.write_all(&chunk)
                .map_err(|_| CommandError::new("update_download_failed"))?;
            downloaded += chunk.len() as u64;
            let _ = app.emit(PROGRESS_EVENT, DownloadProgress { downloaded, total });
        }

        file.sync_all()
            .map_err(|_| CommandError::new("update_download_failed"))?;
        drop(file);
        std::fs::rename(&partial, &target)
            .map_err(|_| CommandError::new("update_download_failed"))?;
        let _ = app.emit(DOWNLOADED_EVENT, ());

        Ok(target)
    }

    /// Hands the APK to `ApkInstaller.kt`, which drives `PackageInstaller` and
    /// surfaces the system confirmation dialog. Returns the Kotlin status
    /// string: `installing`, `permission_required` or `failed`. The installer's
    /// own verdict arrives later on `INSTALL_STATUS_EVENT`.
    fn install_apk(path: &std::path::Path) -> Result<String, CommandError> {
        let path = path
            .to_str()
            .ok_or_else(|| CommandError::new("update_install_failed"))?;

        let context = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(context.vm().cast()) }
            .map_err(|_| CommandError::new("update_install_failed"))?;
        let mut env = vm
            .attach_current_thread()
            .map_err(|_| CommandError::new("update_install_failed"))?;

        // `ndk_context` owns a global ref to the Android context. A `JObject`
        // does not release the reference it wraps, so borrowing it here is safe
        // and the global ref stays alive for the next call.
        let activity = unsafe { jni::objects::JObject::from_raw(context.context().cast()) };
        let apk_path = env
            .new_string(path)
            .map_err(|_| CommandError::new("update_install_failed"))?;

        let result = env.call_static_method(
            "com/betteraimaira/app/ApkInstaller",
            "install",
            "(Landroid/content/Context;Ljava/lang/String;)Ljava/lang/String;",
            &[(&activity).into(), (&apk_path).into()],
        );

        let Ok(value) = result else {
            let _ = env.exception_clear();
            return Err(CommandError::new("update_install_failed"));
        };

        let status = value
            .l()
            .map_err(|_| CommandError::new("update_install_failed"))?;
        let status = jni::objects::JString::from(status);
        let status = env
            .get_string(&status)
            .map_err(|_| CommandError::new("update_install_failed"))?;

        Ok(status.into())
    }
}

// ---------------------------------------------------------------------------
// iOS: AltStore / SideStore source check
// ---------------------------------------------------------------------------

#[cfg(target_os = "ios")]
mod platform {
    use super::*;
    use tauri_plugin_opener::OpenerExt;

    /// Subset of the AltStore source format that the check needs.
    #[derive(Deserialize)]
    struct AltStoreSource {
        apps: Vec<AltStoreApp>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AltStoreApp {
        bundle_identifier: String,
        #[serde(default)]
        versions: Vec<AltStoreVersion>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AltStoreVersion {
        version: String,
        #[serde(default)]
        date: Option<String>,
        #[serde(default)]
        localized_description: Option<String>,
        #[serde(default)]
        download_url: Option<String>,
    }

    /// AltStore and SideStore both take a source URL on this deep link; the one
    /// that is installed answers it. AltStore is tried first.
    fn add_source_link(channel: UpdateChannel) -> String {
        let source = altstore_source_url(channel);
        let source = source.trim_start_matches("https://");
        format!("altstore://source?url={source}")
    }

    pub(super) async fn check(
        app: &AppHandle,
        channel: UpdateChannel,
    ) -> Result<UpdateInfo, CommandError> {
        let current = app.package_info().version.clone();
        let bundle_id = app.config().identifier.clone();

        let client = reqwest::Client::builder()
            .timeout(MANIFEST_TIMEOUT)
            .build()
            .map_err(|_| CommandError::new("update_check_failed"))?;
        let response = client
            .get(altstore_source_url(channel))
            .send()
            .await
            .map_err(|_| CommandError::new("update_check_failed"))?;
        if !response.status().is_success() {
            return Err(CommandError::new("update_check_failed"));
        }
        let body = response
            .text()
            .await
            .map_err(|_| CommandError::new("update_check_failed"))?;
        let source = serde_json::from_str::<AltStoreSource>(&body)
            .map_err(|_| CommandError::new("update_manifest_invalid"))?;

        // The source may list several apps; only this bundle identifier counts.
        let newest = source
            .apps
            .into_iter()
            .find(|entry| entry.bundle_identifier == bundle_id)
            .and_then(|entry| {
                entry
                    .versions
                    .into_iter()
                    .filter_map(|version| {
                        semver::Version::parse(version.version.trim_start_matches('v'))
                            .ok()
                            .map(|parsed| (parsed, version))
                    })
                    .max_by(|left, right| left.0.cmp(&right.0))
            });

        let Some((remote_version, entry)) = newest else {
            return Ok(UpdateInfo::up_to_date(
                current.to_string(),
                UpdateDelivery::AltStore,
                channel,
            ));
        };

        if remote_version <= current {
            return Ok(UpdateInfo::up_to_date(
                current.to_string(),
                UpdateDelivery::AltStore,
                channel,
            ));
        }

        Ok(UpdateInfo {
            available: true,
            current_version: current.to_string(),
            latest_version: Some(entry.version),
            notes: entry.localized_description,
            published_at: entry.date,
            delivery: UpdateDelivery::AltStore,
            channel,
            download_url: entry.download_url,
            store_url: Some(add_source_link(channel)),
        })
    }

    /// iOS cannot install an IPA from inside the app: AltStore does it. The
    /// deep link opens the store on this source so the update is one tap away.
    pub(super) async fn install(
        app: &AppHandle,
        channel: UpdateChannel,
    ) -> Result<InstallOutcome, CommandError> {
        app.opener()
            .open_url(add_source_link(channel), None::<&str>)
            .map_err(|_| CommandError::new("update_store_unavailable"))?;

        Ok(InstallOutcome {
            handed_off: true,
            permission_required: false,
        })
    }
}
