use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use zeroize::Zeroize;

use crate::aimaira;
use crate::credentials;
use crate::downloads;
use crate::error::CommandError;
use crate::grade_sync::{GradeSyncResult, GradeSyncStore};
use crate::portal_store::{schedule_range_key, PortalStore};
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
    /// Whether anything was ever stored for this account. The startup screen
    /// reads it to decide between opening the app on last known content and
    /// blocking on a portal it cannot reach.
    has_snapshots: bool,
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
    stale: bool,
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
    /// The name the view proposes. Sanitised before it reaches the filesystem.
    filename: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDownloadResult {
    /// Absolute path of the saved file, so the view can say where it went.
    path: String,
    /// False when the file is on disk but the system refused to display it.
    opened: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireDetailRequest {
    response_path: String,
}

/// Where a portal page came from, so the caller knows whether it still has to
/// be written to disk.
enum PortalPageSource {
    /// A still-valid copy from the session cache. It was persisted when it was
    /// first fetched, so there is nothing left to write.
    Memory(aimaira::PortalPage),
    Portal(aimaira::PortalPage),
}

fn account_key(portal_url: &url::Url, username: &str) -> String {
    aimaira::stable_hash_hex(&[portal_url.as_str(), username])
}

/// The account a snapshot belongs to, from the session when one is open and
/// from the saved identity otherwise. That fallback is the point of the whole
/// feature: a cold start with no network has no session, and that is exactly
/// when the stored snapshots have to be found.
async fn resolve_account_key(state: &State<'_, SessionState>) -> Option<String> {
    if let Some(key) = session_account_key(state) {
        return Some(key);
    }
    let (portal_url, username) = tauri::async_runtime::spawn_blocking(credentials::load_identity)
        .await
        .ok()?
        .ok()??;
    Some(account_key(&portal_url, &username))
}

fn session_account_key(state: &State<'_, SessionState>) -> Option<String> {
    let guard = state.0.lock().ok()?;
    let session = guard.as_ref()?;
    Some(account_key(&session.portal_url, &session.username))
}

fn has_session(state: &State<'_, SessionState>) -> bool {
    state.0.lock().map(|guard| guard.is_some()).unwrap_or(false)
}

/// Whether a failed read may be answered from disk instead.
///
/// Everything may, except a session the portal itself rejected. Both cases
/// arrive here as `session_expired` — one because no session is open at all,
/// which is the cold offline start this feature exists for, the other because
/// the portal redirected a live session to its login page. Only the first is a
/// candidate for a snapshot: replaying one for the second would leave the
/// reader on data frozen forever, with nothing on screen offering the sign-in
/// that `DESIGN.md` makes the required action for an expired session.
fn may_serve_snapshot(had_session: bool, error: &CommandError) -> bool {
    !(had_session && error.code == "session_expired")
}

/// `has_snapshots` only ever widens what the startup screen may offer, so a
/// storage failure answers "nothing stored" instead of failing the call that
/// screen needs in order to paint itself.
async fn describe_identity(
    portal_store: &State<'_, PortalStore>,
    portal_url: url::Url,
    username: String,
) -> SavedIdentityInfo {
    let store = portal_store.inner().clone();
    let key = account_key(&portal_url, &username);
    let has_snapshots = tauri::async_runtime::spawn_blocking(move || store.has_snapshots(&key))
        .await
        .map(|stored| stored.unwrap_or(false))
        .unwrap_or(false);

    SavedIdentityInfo {
        portal_url: portal_url.to_string(),
        username,
        has_snapshots,
    }
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
pub async fn saved_identity(
    portal_store: State<'_, PortalStore>,
) -> Result<Option<SavedIdentityInfo>, CommandError> {
    let identity = tauri::async_runtime::spawn_blocking(credentials::load_identity)
        .await
        .map_err(|_| CommandError::new("credential_store"))?
        .map_err(|_| CommandError::new("credential_store"))?;

    let Some((portal_url, username)) = identity else {
        return Ok(None);
    };
    Ok(Some(
        describe_identity(&portal_store, portal_url, username).await,
    ))
}

#[tauri::command]
pub async fn restore_session(
    state: State<'_, SessionState>,
    portal_store: State<'_, PortalStore>,
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

    let identity = describe_identity(
        &portal_store,
        saved.portal_url.clone(),
        saved.username.clone(),
    )
    .await;
    let mut password = saved.password;
    let authenticated =
        aimaira::authenticate(saved.portal_url, &saved.username, &password, true).await;
    password.zeroize();

    let authenticated = match authenticated {
        Ok(authenticated) => authenticated,
        Err(error) if error.code == "invalid_credentials" => {
            // Only the password: the portal disproved it, not the account. The
            // identity below is handed back to pre-fill the login form, and it
            // is also what names the snapshots already on disk.
            let _ = tauri::async_runtime::spawn_blocking(credentials::clear_saved_password).await;
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
    portal_store: State<'_, PortalStore>,
    request: ScheduleRequest,
) -> Result<ScheduleResult, CommandError> {
    let start = request.start.trim();
    if start.is_empty() || !(1..=42).contains(&request.duration) {
        return Err(CommandError::new("invalid_schedule_range"));
    }
    let range_key = schedule_range_key(start, request.duration);
    let had_session = has_session(&state);

    match load_schedule_from_portal(&state, start, request.duration).await {
        Ok(result) => {
            persist_schedule(&state, &portal_store, &range_key, &result).await;
            Ok(result)
        }
        Err(error) if may_serve_snapshot(had_session, &error) => {
            stored_schedule(&state, &portal_store, &range_key)
                .await
                .ok_or(error)
        }
        Err(error) => Err(error),
    }
}

async fn load_schedule_from_portal(
    state: &State<'_, SessionState>,
    start: &str,
    duration: u8,
) -> Result<ScheduleResult, CommandError> {
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
        start,
        duration,
    )
    .await?;

    Ok(ScheduleResult {
        events,
        fetched_at: aimaira::current_timestamp_millis()?,
        stale: false,
    })
}

async fn persist_schedule(
    state: &State<'_, SessionState>,
    portal_store: &State<'_, PortalStore>,
    range_key: &str,
    result: &ScheduleResult,
) {
    let Some(account_key) = resolve_account_key(state).await else {
        return;
    };
    let store = portal_store.inner().clone();
    let range_key = range_key.to_owned();
    let events = result.events.clone();
    let fetched_at = result.fetched_at;
    // A snapshot that fails to save is not worth failing a range the caller
    // already holds, so the outcome is dropped rather than propagated.
    let _ = tauri::async_runtime::spawn_blocking(move || {
        store.save_schedule(&account_key, &range_key, &events, fetched_at)
    })
    .await;
}

async fn stored_schedule(
    state: &State<'_, SessionState>,
    portal_store: &State<'_, PortalStore>,
    range_key: &str,
) -> Option<ScheduleResult> {
    let account_key = resolve_account_key(state).await?;
    let store = portal_store.inner().clone();
    let range_key = range_key.to_owned();
    let stored =
        tauri::async_runtime::spawn_blocking(move || store.load_schedule(&account_key, &range_key))
            .await
            .ok()?
            .ok()??;

    Some(ScheduleResult {
        events: stored.events,
        fetched_at: stored.fetched_at,
        stale: true,
    })
}

#[tauri::command]
pub async fn get_portal_resource(
    state: State<'_, SessionState>,
    portal_store: State<'_, PortalStore>,
    resource: aimaira::PortalResource,
    force: Option<bool>,
) -> Result<aimaira::PortalPage, CommandError> {
    load_cached_portal_resource(&state, &portal_store, resource, force.unwrap_or(false)).await
}

/// Three tiers, in order: the session cache, the portal, then the on-disk
/// snapshot. The last one is what lets the app open with no network at all; it
/// keeps the timestamp of the fetch it came from and is flagged `stale`.
async fn load_cached_portal_resource(
    state: &State<'_, SessionState>,
    portal_store: &State<'_, PortalStore>,
    resource: aimaira::PortalResource,
    force: bool,
) -> Result<aimaira::PortalPage, CommandError> {
    let had_session = has_session(state);

    match fetch_portal_resource(state, resource, force).await {
        Ok(PortalPageSource::Memory(page)) => Ok(page),
        Ok(PortalPageSource::Portal(page)) => {
            persist_portal_page(state, portal_store, &page).await;
            Ok(page)
        }
        Err(error) if may_serve_snapshot(had_session, &error) => {
            stored_portal_page(state, portal_store, resource)
                .await
                .ok_or(error)
        }
        Err(error) => Err(error),
    }
}

async fn persist_portal_page(
    state: &State<'_, SessionState>,
    portal_store: &State<'_, PortalStore>,
    page: &aimaira::PortalPage,
) {
    let Some(account_key) = resolve_account_key(state).await else {
        return;
    };
    let store = portal_store.inner().clone();
    let page = page.clone();
    // A snapshot that fails to save is not worth failing a page the caller
    // already holds, so the outcome is dropped rather than propagated.
    let _ =
        tauri::async_runtime::spawn_blocking(move || store.save_portal_page(&account_key, &page))
            .await;
}

async fn stored_portal_page(
    state: &State<'_, SessionState>,
    portal_store: &State<'_, PortalStore>,
    resource: aimaira::PortalResource,
) -> Option<aimaira::PortalPage> {
    let account_key = resolve_account_key(state).await?;
    let store = portal_store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || store.load_portal_page(&account_key, resource))
        .await
        .ok()?
        .ok()?
}

/// Stamps the request that is about to leave, so the page it brings back can be
/// matched to it rather than to whatever the session has done since.
///
/// Split out of `fetch_portal_resource`, together with the check below, because
/// the command they serve needs a Tauri `State` no unit test can build — and
/// what they enforce (a result never lands in a session that did not ask for
/// it) is the one part worth pinning.
fn issue_request_version(session: &mut AimairaSession, resource: aimaira::PortalResource) -> u64 {
    session
        .portal_cache_versions
        .entry(resource)
        .and_modify(|version| *version += 1)
        .or_insert(1)
        .to_owned()
}

/// Whether the page that just arrived is still the one the session is waiting
/// for. A logout, a sign-in as somebody else, or a newer read of the same
/// resource all make it stale, and caching a stale page would serve one
/// account's portal to the session that replaced it.
fn answers_the_pending_request(
    session: &AimairaSession,
    resource: aimaira::PortalResource,
    session_id: u64,
    portal_url: &url::Url,
    request_version: u64,
) -> bool {
    session.id == session_id
        && session.portal_url == *portal_url
        && session.portal_cache_versions.get(&resource) == Some(&request_version)
}

async fn fetch_portal_resource(
    state: &State<'_, SessionState>,
    resource: aimaira::PortalResource,
    force: bool,
) -> Result<PortalPageSource, CommandError> {
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
                    return Ok(PortalPageSource::Memory(cached.page.clone()));
                }
            }
        }
        let request_version = issue_request_version(session, resource);
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
        if answers_the_pending_request(session, resource, session_id, &portal_url, request_version)
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
    Ok(PortalPageSource::Portal(page))
}

#[tauri::command]
pub async fn sync_grades(
    state: State<'_, SessionState>,
    grade_store: State<'_, GradeSyncStore>,
    portal_store: State<'_, PortalStore>,
    force: Option<bool>,
) -> Result<GradeSyncResult, CommandError> {
    let account_key = resolve_account_key(&state)
        .await
        .ok_or_else(|| CommandError::new("session_expired"))?;
    let page = load_cached_portal_resource(
        &state,
        &portal_store,
        aimaira::PortalResource::Grades,
        force.unwrap_or(false),
    )
    .await?;

    // A page replayed from disk says nothing about what the portal holds now,
    // so the stored rows are read back as they are. Running them through the
    // diff would announce, as new, grades the reader has already been shown.
    if page.stale {
        let store = grade_store.inner().clone();
        return run_grade_store(move || store.stored_snapshot(&account_key)).await;
    }

    // The whole history is what tells us the page parsed at all: the most
    // recent school year is legitimately empty in the days after it opens.
    if page.markup_recognized && aimaira::extract_grades(&page).is_empty() {
        return Err(CommandError::new("grades_invalid_response"));
    }
    let latest_grades = aimaira::extract_latest_grades(&page);
    let store = grade_store.inner().clone();
    run_grade_store(move || store.persist(&account_key, latest_grades)).await
}

async fn run_grade_store<F>(operation: F) -> Result<GradeSyncResult, CommandError>
where
    F: FnOnce() -> Result<GradeSyncResult, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|_| CommandError::new("grade_storage_unavailable"))?
        .map_err(|_| CommandError::new("grade_storage_unavailable"))
}

/// Saves the PDF from Rust rather than handing the bytes to the webview: no
/// engine the app ships on downloads a `blob:` URL reliably, so the button used
/// to finish loading and leave nothing behind. See `crate::downloads`.
#[tauri::command]
pub async fn download_portal_document(
    app: AppHandle,
    state: State<'_, SessionState>,
    request: DocumentRequest,
) -> Result<DocumentDownloadResult, CommandError> {
    let (client, portal_url) = state.with_session(|s| (s.client.clone(), s.portal_url.clone()))?;
    let body = aimaira::download_portal_document(&client, &portal_url, request.request_path.trim())
        .await?;
    let saved = downloads::save_document(&app, &request.filename, &body)?;

    // The document is already safe on disk; failing to display it is worth
    // reporting but must not read as a failed download.
    let opened = app
        .opener()
        .open_path(saved.to_string_lossy(), None::<&str>)
        .is_ok();

    Ok(DocumentDownloadResult {
        path: saved.to_string_lossy().into_owned(),
        opened,
    })
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{answers_the_pending_request, issue_request_version, may_serve_snapshot};
    use crate::aimaira::{PlanningSettings, PortalResource};
    use crate::error::CommandError;
    use crate::state::AimairaSession;

    fn session(id: u64, portal: &str) -> AimairaSession {
        AimairaSession {
            id,
            client: reqwest::Client::new(),
            portal_url: url::Url::parse(portal).expect("a valid portal URL"),
            username: "student".to_owned(),
            planning: PlanningSettings::default(),
            portal_cache: HashMap::new(),
            portal_cache_versions: HashMap::new(),
        }
    }

    #[test]
    fn a_portal_that_rejects_a_live_session_is_never_answered_from_disk() {
        let expired = CommandError::new("session_expired");

        // No session open: the cold offline start this feature exists for.
        assert!(may_serve_snapshot(false, &expired));
        // A live session the portal just refused. Replaying a snapshot here
        // would hide the expiry and strand the reader on frozen data.
        assert!(!may_serve_snapshot(true, &expired));
    }

    #[test]
    fn an_unreachable_portal_is_answered_from_disk_either_way() {
        for code in ["portal_unreachable", "grades_unavailable", "internal_error"] {
            let error = CommandError::new(code);
            assert!(may_serve_snapshot(true, &error), "{code} with a session");
            assert!(may_serve_snapshot(false, &error), "{code} without one");
        }
    }

    #[test]
    fn a_page_that_arrives_after_a_logout_is_never_cached_for_whoever_signed_in_next() {
        let mut first = session(1, "https://portal.example.test/");
        let version = issue_request_version(&mut first, PortalResource::Grades);
        let portal_url = first.portal_url.clone();

        // The reader logs out mid-flight and someone else signs in. Only the
        // request is still in the air; the session it belonged to is gone.
        let replacement = session(2, "https://portal.example.test/");

        assert!(!answers_the_pending_request(
            &replacement,
            PortalResource::Grades,
            first.id,
            &portal_url,
            version,
        ));
    }

    #[test]
    fn the_same_account_pointed_at_another_portal_does_not_inherit_the_answer() {
        let mut session = session(1, "https://portal.example.test/");
        let version = issue_request_version(&mut session, PortalResource::Profile);
        let old_portal = session.portal_url.clone();
        session.portal_url = url::Url::parse("https://other.example.test/").unwrap();

        assert!(!answers_the_pending_request(
            &session,
            PortalResource::Profile,
            session.id,
            &old_portal,
            version,
        ));
    }

    #[test]
    fn only_the_newest_read_of_a_resource_writes_itself_into_the_cache() {
        let mut session = session(1, "https://portal.example.test/");
        let portal_url = session.portal_url.clone();

        let first = issue_request_version(&mut session, PortalResource::Absences);
        let second = issue_request_version(&mut session, PortalResource::Absences);
        assert_ne!(first, second);

        // A forced refresh overtakes a slow read already in flight. The slow
        // one coming back last must not put the older page back on top.
        assert!(!answers_the_pending_request(
            &session,
            PortalResource::Absences,
            session.id,
            &portal_url,
            first,
        ));
        assert!(answers_the_pending_request(
            &session,
            PortalResource::Absences,
            session.id,
            &portal_url,
            second,
        ));
    }

    #[test]
    fn each_resource_is_versioned_on_its_own() {
        let mut session = session(1, "https://portal.example.test/");
        let portal_url = session.portal_url.clone();

        let grades = issue_request_version(&mut session, PortalResource::Grades);
        // Reading another resource must not invalidate the grades read that is
        // still in flight — the shell loads all five at once.
        issue_request_version(&mut session, PortalResource::Documents);

        assert!(answers_the_pending_request(
            &session,
            PortalResource::Grades,
            session.id,
            &portal_url,
            grades,
        ));
    }
}
