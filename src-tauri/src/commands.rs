use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::State;
use zeroize::Zeroize;

use crate::aimaira;
use crate::credentials;
use crate::error::CommandError;
use crate::grade_sync::{GradeSyncResult, GradeSyncStore};
use crate::state::{AimairaSession, PortalCacheEntry, SessionState};

const PORTAL_CACHE_TTL: Duration = Duration::from_secs(5 * 60);
static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    portal_url: String,
    username: String,
    password: String,
    remember: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResult {
    portal_url: String,
    username: String,
    credentials_saved: bool,
    sundays_visible: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalInfo {
    portal_url: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedIdentityInfo {
    portal_url: String,
    username: String,
}

/// `restore_session` used to answer `null` for both "nothing was saved" and
/// "what was saved is no longer accepted", which left the startup screen unable
/// to explain itself. The status separates the two, and the identity travels
/// along so a rejected restore can hand the login form a filled account.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    status: &'static str,
    session: Option<LoginResult>,
    identity: Option<SavedIdentityInfo>,
}

impl RestoreResult {
    const NO_CREDENTIALS: &'static str = "no_credentials";
    const CREDENTIALS_REJECTED: &'static str = "credentials_rejected";
    const RESTORED: &'static str = "restored";
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleRequest {
    #[serde(alias = "startDate")]
    start: String,
    #[serde(alias = "durationDays")]
    duration: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleResult {
    events: Vec<aimaira::CalendarEvent>,
    fetched_at: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningSettingsResult {
    sundays_visible: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRequest {
    request_path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireDetailRequest {
    response_path: String,
}

fn account_key(portal_url: &url::Url, username: &str) -> String {
    aimaira::stable_hash_hex(&[portal_url.as_str(), username])
}

#[tauri::command]
pub fn normalize_portal_url(portal_url: String) -> Result<PortalInfo, CommandError> {
    let normalized = aimaira::normalize_portal_url(&portal_url)?;
    Ok(PortalInfo {
        portal_url: normalized.to_string(),
    })
}

#[tauri::command]
pub async fn login(
    state: State<'_, SessionState>,
    request: LoginRequest,
) -> Result<LoginResult, CommandError> {
    let portal_url = aimaira::normalize_portal_url(&request.portal_url)?;
    let username = request.username.trim().to_owned();
    if username.is_empty() || request.password.is_empty() {
        return Err(CommandError::new("missing_credentials"));
    }

    let mut password = request.password;
    let authenticated =
        match aimaira::authenticate(portal_url, &username, &password, request.remember).await {
            Ok(authenticated) => authenticated,
            Err(error) => {
                password.zeroize();
                return Err(error);
            }
        };
    let credentials_portal = authenticated.portal_url.clone();
    let credentials_username = username.clone();
    let remember = request.remember;
    let credentials_saved = tauri::async_runtime::spawn_blocking(move || {
        let saved = if remember {
            credentials::save_password(&credentials_portal, &credentials_username, &password)
                .is_ok()
        } else {
            let _ = credentials::clear_saved_credentials();
            false
        };
        password.zeroize();
        saved
    })
    .await
    .unwrap_or(false);

    complete_login(state, authenticated, username, credentials_saved)
}

/// Reads the saved identity without touching the password entry, so the startup
/// screen can decide between a restore wait and the login form on its first
/// paint instead of flashing the form.
#[tauri::command]
pub async fn saved_identity() -> Result<Option<SavedIdentityInfo>, CommandError> {
    let identity = tauri::async_runtime::spawn_blocking(credentials::load_identity)
        .await
        .map_err(|_| CommandError::new("credential_store"))?
        .map_err(|_| CommandError::new("credential_store"))?;

    Ok(identity.map(|(portal_url, username)| SavedIdentityInfo {
        portal_url: portal_url.to_string(),
        username,
    }))
}

#[tauri::command]
pub async fn restore_session(
    state: State<'_, SessionState>,
) -> Result<RestoreResult, CommandError> {
    let Some(saved) = tauri::async_runtime::spawn_blocking(credentials::load_credentials)
        .await
        .map_err(|_| CommandError::new("credential_store"))?
        .map_err(|_| CommandError::new("credential_store"))?
    else {
        return Ok(RestoreResult {
            status: RestoreResult::NO_CREDENTIALS,
            session: None,
            identity: None,
        });
    };

    let identity = SavedIdentityInfo {
        portal_url: saved.portal_url.to_string(),
        username: saved.username.clone(),
    };
    let mut password = saved.password;
    let authenticated =
        aimaira::authenticate(saved.portal_url, &saved.username, &password, true).await;
    password.zeroize();

    let authenticated = match authenticated {
        Ok(authenticated) => authenticated,
        Err(error) if error.code == "invalid_credentials" => {
            let _ = tauri::async_runtime::spawn_blocking(credentials::clear_saved_credentials).await;
            return Ok(RestoreResult {
                status: RestoreResult::CREDENTIALS_REJECTED,
                session: None,
                identity: Some(identity),
            });
        }
        Err(error) => return Err(error),
    };

    let session = complete_login(state, authenticated, saved.username, true)?;
    Ok(RestoreResult {
        status: RestoreResult::RESTORED,
        session: Some(session),
        identity: Some(identity),
    })
}

fn complete_login(
    state: State<'_, SessionState>,
    authenticated: aimaira::AuthenticatedSession,
    username: String,
    credentials_saved: bool,
) -> Result<LoginResult, CommandError> {
    let normalized_portal_url = authenticated.portal_url.to_string();
    let session = AimairaSession {
        id: NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed),
        client: authenticated.client,
        portal_url: authenticated.portal_url,
        username: username.clone(),
        planning: aimaira::PlanningSettings::default(),
        portal_cache: HashMap::new(),
        portal_cache_versions: HashMap::new(),
    };
    *state
        .0
        .lock()
        .map_err(|_| CommandError::new("internal_error"))? = Some(session);

    Ok(LoginResult {
        portal_url: normalized_portal_url,
        username,
        credentials_saved,
        sundays_visible: false,
    })
}

/// Always re-reads the settings from the portal and refreshes the cached copy on
/// the session, so the caller never has to ask for a refresh separately.
#[tauri::command]
pub async fn get_planning_settings(
    state: State<'_, SessionState>,
) -> Result<PlanningSettingsResult, CommandError> {
    let (client, portal_url, session_id) =
        state.with_session(|s| (s.client.clone(), s.portal_url.clone(), s.id))?;
    let planning = aimaira::load_planning_settings(&client, &portal_url).await?;
    let result = PlanningSettingsResult {
        sundays_visible: planning.sundays_visible,
    };
    if let Ok(mut guard) = state.0.lock() {
        if let Some(session) = guard
            .as_mut()
            .filter(|s| s.id == session_id && s.portal_url == portal_url)
        {
            session.planning = planning;
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn get_schedule(
    state: State<'_, SessionState>,
    request: ScheduleRequest,
) -> Result<ScheduleResult, CommandError> {
    if request.start.trim().is_empty() || !(1..=42).contains(&request.duration) {
        return Err(CommandError::new("invalid_schedule_range"));
    }

    let (client, portal_url, tempo_base_url) = state.with_session(|s| {
        (
            s.client.clone(),
            s.portal_url.clone(),
            s.planning.tempo_base_url.clone(),
        )
    })?;
    let events = aimaira::load_calendar_events(
        &client,
        &portal_url,
        tempo_base_url.as_ref(),
        request.start.trim(),
        request.duration,
    )
    .await?;
    let fetched_at = aimaira::current_timestamp_millis()?;

    Ok(ScheduleResult { events, fetched_at })
}

#[tauri::command]
pub async fn get_portal_resource(
    state: State<'_, SessionState>,
    resource: aimaira::PortalResource,
    force: Option<bool>,
) -> Result<aimaira::PortalPage, CommandError> {
    load_cached_portal_resource(&state, resource, force.unwrap_or(false)).await
}

async fn load_cached_portal_resource(
    state: &State<'_, SessionState>,
    resource: aimaira::PortalResource,
    force: bool,
) -> Result<aimaira::PortalPage, CommandError> {
    let (client, portal_url, session_id, request_version) = {
        let mut session = state
            .0
            .lock()
            .map_err(|_| CommandError::new("internal_error"))?;
        let session = session
            .as_mut()
            .ok_or_else(|| CommandError::new("session_expired"))?;
        if !force {
            if let Some(cached) = session.portal_cache.get(&resource) {
                if cached.expires_at > Instant::now() {
                    return Ok(cached.page.clone());
                }
            }
        }
        let request_version = session
            .portal_cache_versions
            .entry(resource)
            .and_modify(|version| *version += 1)
            .or_insert(1)
            .to_owned();
        (
            session.client.clone(),
            session.portal_url.clone(),
            session.id,
            request_version,
        )
    };

    let page = aimaira::load_portal_resource(&client, &portal_url, resource).await?;
    let mut session = state
        .0
        .lock()
        .map_err(|_| CommandError::new("internal_error"))?;
    if let Some(session) = session.as_mut() {
        if session.id == session_id
            && session.portal_url == portal_url
            && session.portal_cache_versions.get(&resource) == Some(&request_version)
        {
            session.portal_cache.insert(
                resource,
                PortalCacheEntry {
                    page: page.clone(),
                    expires_at: Instant::now() + PORTAL_CACHE_TTL,
                },
            );
        }
    }
    Ok(page)
}

#[tauri::command]
pub async fn sync_grades(
    state: State<'_, SessionState>,
    grade_store: State<'_, GradeSyncStore>,
    force: Option<bool>,
) -> Result<GradeSyncResult, CommandError> {
    let (portal_url, username) =
        state.with_session(|s| (s.portal_url.clone(), s.username.clone()))?;
    let page = load_cached_portal_resource(&state, aimaira::PortalResource::Grades, force.unwrap_or(false)).await?;
    let grades = aimaira::extract_grades(&page);
    let latest_grades = aimaira::extract_latest_grades(&page);
    if page.markup_recognized && grades.is_empty() {
        return Err(CommandError::new("grades_invalid_response"));
    }
    let store = grade_store.inner().clone();
    let account_key = account_key(&portal_url, &username);
    tauri::async_runtime::spawn_blocking(move || {
        let mut result = store.sync(&account_key, grades)?;
        result.grades = latest_grades;
        Ok::<GradeSyncResult, String>(result)
    })
        .await
        .map_err(|_| CommandError::new("grade_storage_unavailable"))?
        .map_err(|_| CommandError::new("grade_storage_unavailable"))
}

#[tauri::command]
pub async fn mark_grade_alerts_read(
    state: State<'_, SessionState>,
    grade_store: State<'_, GradeSyncStore>,
) -> Result<(), CommandError> {
    let (portal_url, username) =
        state.with_session(|s| (s.portal_url.clone(), s.username.clone()))?;
    let store = grade_store.inner().clone();
    let account_key = account_key(&portal_url, &username);
    tauri::async_runtime::spawn_blocking(move || store.mark_alerts_read(&account_key))
        .await
        .map_err(|_| CommandError::new("grade_storage_unavailable"))?
        .map_err(|_| CommandError::new("grade_storage_unavailable"))
}

#[tauri::command]
pub async fn download_portal_document(
    state: State<'_, SessionState>,
    request: DocumentRequest,
) -> Result<tauri::ipc::Response, CommandError> {
    let (client, portal_url) =
        state.with_session(|s| (s.client.clone(), s.portal_url.clone()))?;
    let body = aimaira::download_portal_document(&client, &portal_url, request.request_path.trim())
        .await?;
    Ok(tauri::ipc::Response::new(body))
}

#[tauri::command]
pub async fn get_questionnaire_detail(
    state: State<'_, SessionState>,
    request: QuestionnaireDetailRequest,
) -> Result<aimaira::QuestionnaireDetail, CommandError> {
    let (client, portal_url) =
        state.with_session(|session| (session.client.clone(), session.portal_url.clone()))?;
    aimaira::load_questionnaire_detail(&client, &portal_url, request.response_path.trim()).await
}

#[tauri::command]
pub async fn logout(state: State<'_, SessionState>) -> Result<(), CommandError> {
    tauri::async_runtime::spawn_blocking(credentials::clear_saved_credentials)
        .await
        .map_err(|_| CommandError::new("credential_store"))?
        .map_err(|_| CommandError::new("credential_store"))?;
    *state
        .0
        .lock()
        .map_err(|_| CommandError::new("internal_error"))? = None;
    Ok(())
}
