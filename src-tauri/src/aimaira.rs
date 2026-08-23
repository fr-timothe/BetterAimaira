use std::sync::Arc;
use std::time::Duration;

use reqwest::{cookie::Jar, Client, Url};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::error::CommandError;

mod portal;

pub use portal::{
    download_portal_document, extract_grades, extract_latest_grades, load_portal_resource,
    load_questionnaire_detail, Grade, PortalPage, PortalResource, QuestionnaireDetail,
};

const USER_AGENT: &str = "BetterAimaira/0.1";

pub struct AuthenticatedSession {
    pub client: Client,
    pub portal_url: Url,
}

/// Planning settings the portal inlines in its own `/Calendar` page.
#[derive(Debug, Default, Clone)]
pub struct PlanningSettings {
    pub tempo_base_url: Option<Url>,
    pub sundays_visible: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AimairaCalendarEvent {
    id: String,
    is_seance: bool,
    display_tempo_link: bool,
    debut: String,
    fin: String,
    #[serde(default)]
    planification: String,
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    commentaire_externe: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub starts_at: String,
    pub ends_at: String,
    pub planification: String,
    pub description: String,
    pub kind: String,
    pub external_comment: String,
    pub tempo_url: Option<String>,
}

pub fn normalize_portal_url(input: &str) -> Result<Url, CommandError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(CommandError::new("invalid_portal_url"));
    }

    let candidate = if trimmed.contains("://") {
        trimmed.to_owned()
    } else {
        format!("https://{trimmed}")
    };

    let mut url = Url::parse(&candidate).map_err(|_| CommandError::new("invalid_portal_url"))?;

    if url.scheme() != "https" {
        return Err(CommandError::new("insecure_portal_url"));
    }
    if url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return Err(CommandError::new("invalid_portal_url"));
    }

    url.set_path("/");
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

pub async fn authenticate(
    portal_url: Url,
    username: &str,
    password: &str,
    remember: bool,
) -> Result<AuthenticatedSession, CommandError> {
    let cookie_jar = Arc::new(Jar::default());
    let client = Client::builder()
        .cookie_provider(cookie_jar)
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .build()?;

    let login_url = portal_url
        .join("login?ReturnUrl=%2F")
        .map_err(|_| CommandError::new("invalid_portal_url"))?;
    let login_page = client.get(login_url).send().await?;
    if !login_page.status().is_success() {
        return Err(CommandError::new("portal_unreachable"));
    }

    let login_html = login_page.text().await?;
    let token =
        extract_csrf_token(&login_html).ok_or_else(|| CommandError::new("portal_not_aimaira"))?;
    let submit_url = portal_url
        .join("User/LoginPost?ReturnUrl=%2F")
        .map_err(|_| CommandError::new("invalid_portal_url"))?;

    let mut form = vec![
        ("UserName", username),
        ("Password", password),
        ("__RequestVerificationToken", token.as_str()),
    ];
    if remember {
        form.push(("remember", "Remember Me"));
    }

    let response = client.post(submit_url).form(&form).send().await?;

    if !response.status().is_success() {
        return Err(CommandError::new("portal_unreachable"));
    }

    let final_url = response.url().clone();
    let response_html = response.text().await?;
    if is_login_page(&final_url, &response_html) {
        return Err(CommandError::new("invalid_credentials"));
    }

    Ok(AuthenticatedSession { client, portal_url })
}

pub async fn load_calendar_events(
    client: &Client,
    portal_url: &Url,
    tempo_base_url: Option<&Url>,
    start: &str,
    duration: u8,
) -> Result<Vec<CalendarEvent>, CommandError> {
    let endpoint = portal_url
        .join("Calendar/LoadEvents")
        .map_err(|_| CommandError::new("invalid_portal_url"))?;
    let duration = duration.to_string();
    let response = client
        .post(endpoint)
        .header("X-Requested-With", "XMLHttpRequest")
        .form(&[("start", start), ("duration", duration.as_str())])
        .send()
        .await
        .map_err(|_| CommandError::new("planning_unavailable"))?;

    if !response.status().is_success() {
        return Err(CommandError::new("planning_unavailable"));
    }

    let final_url = response.url().clone();
    let body = response
        .text()
        .await
        .map_err(|_| CommandError::new("planning_unavailable"))?;
    if is_login_page(&final_url, &body) {
        return Err(CommandError::new("session_expired"));
    }

    parse_calendar_events(tempo_base_url, &body)
}

fn parse_calendar_events(
    tempo_base_url: Option<&Url>,
    body: &str,
) -> Result<Vec<CalendarEvent>, CommandError> {
    // LoadEvents sometimes answers 200 with an empty body. The portal's own script crashes on
    // `JSON.parse("")` there, so treat it as an empty range instead of a parse failure.
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }

    let events: Vec<AimairaCalendarEvent> =
        serde_json::from_str(body).map_err(|_| CommandError::new("planning_invalid_response"))?;
    Ok(events
        .into_iter()
        .filter(|event| !event.debut.is_empty() && !event.fin.is_empty() && event.debut < event.fin)
        .map(|event| normalize_calendar_event(tempo_base_url, event))
        .collect())
}

fn normalize_calendar_event(
    tempo_base_url: Option<&Url>,
    event: AimairaCalendarEvent,
) -> CalendarEvent {
    // The portal builds session links as `urlTempoSeance + Id`, and only when its own
    // `tempoLinkVisible` flag is set. Without that base URL there is no link to expose.
    let tempo_url = tempo_base_url
        .filter(|_| event.is_seance && event.display_tempo_link)
        .map(|base| format!("{base}{}", event.id));

    CalendarEvent {
        id: event.id,
        starts_at: event.debut,
        ends_at: event.fin,
        planification: html_to_text(&event.planification),
        description: html_to_text(&event.description),
        kind: normalize_whitespace(&event.r#type),
        external_comment: html_to_text(&event.commentaire_externe),
        tempo_url,
    }
}

pub async fn load_planning_settings(
    client: &Client,
    portal_url: &Url,
) -> Result<PlanningSettings, CommandError> {
    let endpoint = portal_url
        .join("Calendar")
        .map_err(|_| CommandError::new("invalid_portal_url"))?;
    let response = client
        .get(endpoint)
        .send()
        .await
        .map_err(|_| CommandError::new("planning_unavailable"))?;
    if !response.status().is_success() {
        return Err(CommandError::new("planning_unavailable"));
    }
    let final_url = response.url().clone();
    let body = response
        .text()
        .await
        .map_err(|_| CommandError::new("planning_unavailable"))?;
    if is_login_page(&final_url, &body) {
        return Err(CommandError::new("session_expired"));
    }

    Ok(parse_planning_settings(&body))
}

fn parse_planning_settings(html: &str) -> PlanningSettings {
    let tempo_link_visible = extract_inline_const(html, "tempoLinkVisible")
        .is_some_and(|value| value.eq_ignore_ascii_case("true"));
    let tempo_base_url = tempo_link_visible
        .then(|| extract_inline_const(html, "urlTempoSeance"))
        .flatten()
        .and_then(|value| Url::parse(&value).ok())
        .filter(|url| url.scheme() == "https");

    PlanningSettings {
        tempo_base_url,
        sundays_visible: extract_inline_const(html, "sundaysVisible")
            .is_some_and(|value| value.eq_ignore_ascii_case("true")),
    }
}

/// Reads a `const <name> = '<value>'` literal out of the portal's inline planning script.
fn extract_inline_const(html: &str, name: &str) -> Option<String> {
    let declaration = format!("const {name} = '");
    let start = html.find(&declaration)? + declaration.len();
    let value = &html[start..];
    let end = value.find('\'')?;
    Some(value[..end].to_owned())
}

pub fn current_timestamp_millis() -> Result<u64, CommandError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .map_err(|_| CommandError::new("internal_error"))
}

pub fn stable_hash(values: &[&str]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for value in values {
        for byte in value.as_bytes().iter().chain(std::iter::once(&0)) {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    hash
}

pub fn stable_hash_hex(values: &[&str]) -> String {
    format!("{:016x}", stable_hash(values))
}

pub(crate) fn html_to_text(value: &str) -> String {
    if value.trim().is_empty() {
        return String::new();
    }

    let with_newlines = value
        .replace("<br />", "\n")
        .replace("<br/>", "\n")
        .replace("<br>", "\n")
        .replace("<BR />", "\n")
        .replace("<BR/>", "\n")
        .replace("<BR>", "\n")
        .replace("</p>", "\n")
        .replace("</P>", "\n")
        .replace("</div>", "\n")
        .replace("</DIV>", "\n");

    let document = Html::parse_fragment(&with_newlines);
    let raw_text = document.root_element().text().collect::<Vec<_>>().join("");

    let unescaped = raw_text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">");

    let lines: Vec<&str> = unescaped
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    lines.join("\n")
}

pub(crate) fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_csrf_token(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("input[name='__RequestVerificationToken']").ok()?;
    document
        .select(&selector)
        .find_map(|element| element.value().attr("value"))
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn is_login_page(url: &Url, html: &str) -> bool {
    let path = url.path().to_ascii_lowercase();
    if path == "/login" || path.starts_with("/user/login") {
        return true;
    }

    if !html.contains("<form") && !html.contains("<input") {
        return false;
    }

    let document = Html::parse_document(html);
    is_login_document(&document)
}

fn is_login_document(document: &Html) -> bool {
    let login_form = Selector::parse("form[action*='LoginPost']")
        .ok()
        .is_some_and(|selector| document.select(&selector).next().is_some());
    let has_username = Selector::parse("input[name='UserName']")
        .ok()
        .is_some_and(|selector| document.select(&selector).next().is_some());
    let has_password = Selector::parse("input[name='Password']")
        .ok()
        .is_some_and(|selector| document.select(&selector).next().is_some());

    login_form || (has_username && has_password)
}

#[cfg(test)]
mod tests {
    use super::{
        extract_csrf_token, html_to_text, is_login_page, normalize_portal_url,
        parse_calendar_events, parse_planning_settings,
    };
    use reqwest::Url;

    #[test]
    fn normalizes_deep_portal_urls_to_https_origin() {
        let url =
            normalize_portal_url("  school.myintranet.online/Calendar?view=week#today  ").unwrap();

        assert_eq!(url.as_str(), "https://school.myintranet.online/");
    }

    #[test]
    fn rejects_insecure_or_credentialed_urls() {
        assert_eq!(
            normalize_portal_url("http://school.myintranet.online")
                .unwrap_err()
                .code,
            "insecure_portal_url"
        );
        assert_eq!(
            normalize_portal_url("https://user:secret@example.com")
                .unwrap_err()
                .code,
            "invalid_portal_url"
        );
    }

    #[test]
    fn extracts_asp_net_csrf_token() {
        let html = r#"<form><input name="__RequestVerificationToken" type="hidden" value="token-123" /></form>"#;
        assert_eq!(extract_csrf_token(html).as_deref(), Some("token-123"));
    }

    #[test]
    fn identifies_login_responses_by_path_or_form() {
        let portal = Url::parse("https://example.com/Calendar").unwrap();
        let login = Url::parse("https://example.com/login?ReturnUrl=%2F").unwrap();
        let form = r#"<form action="/User/LoginPost"><input name="UserName" /><input name="Password" /></form>"#;
        let unrelated_token =
            r#"<form><input name="__RequestVerificationToken" value="token" /></form>"#;

        assert!(is_login_page(&login, ""));
        assert!(is_login_page(&portal, form));
        assert!(!is_login_page(&portal, unrelated_token));
        assert!(!is_login_page(&portal, "<main>Calendar</main>"));
        assert!(!is_login_page(&portal, r#"[{"Id":"1"}]"#));
    }

    #[test]
    fn treats_empty_load_events_body_as_a_free_period() {
        assert!(parse_calendar_events(None, "").unwrap().is_empty());
        assert!(parse_calendar_events(None, "  \r\n ").unwrap().is_empty());
        assert_eq!(
            parse_calendar_events(None, "<html>not json</html>")
                .unwrap_err()
                .code,
            "planning_invalid_response"
        );
    }

    #[test]
    fn builds_tempo_url_only_for_linkable_sessions() {
        let tempo = Url::parse("https://school.mytempo.online/user/login").unwrap();
        let body = r#"[
            {"Id":"1","IsSeance":true,"DisplayTempoLink":true,"Debut":"2026-08-17T08:00:00","Fin":"2026-08-17T10:00:00","Planification":"Dev Web","Description":"Salle B204","Type":"Cours","CommentaireExterne":""},
            {"Id":"2","IsSeance":true,"DisplayTempoLink":false,"Debut":"2026-08-17T10:00:00","Fin":"2026-08-17T12:00:00","Planification":"Maths","Description":"","Type":"","CommentaireExterne":""},
            {"Id":"3","IsSeance":true,"DisplayTempoLink":true,"Debut":"2026-08-17T14:00:00","Fin":"2026-08-17T14:00:00","Planification":"Zero length","Description":"","Type":"","CommentaireExterne":""}
        ]"#;

        let linked = parse_calendar_events(Some(&tempo), body).unwrap();
        assert_eq!(linked.len(), 2);
        assert_eq!(
            linked[0].tempo_url.as_deref(),
            Some("https://school.mytempo.online/user/login1")
        );
        assert_eq!(linked[1].tempo_url, None);

        // Without the portal-provided base URL there is no session link to expose.
        let unlinked = parse_calendar_events(None, body).unwrap();
        assert_eq!(unlinked[0].tempo_url, None);
    }

    #[test]
    fn reads_planning_settings_from_the_inline_portal_script() {
        let visible = r#"
            const urlTempoSeance = 'https://school.mytempo.online/user/login' ?? "";
            const tempoLinkVisible = 'True'.toLowerCase() === 'true';
            const sundaysVisible = 'True'.toLowerCase() === 'true';
        "#;
        let hidden = r#"
            const urlTempoSeance = 'https://school.mytempo.online/user/login' ?? "";
            const tempoLinkVisible = 'False'.toLowerCase() === 'true';
            const sundaysVisible = 'False'.toLowerCase() === 'true';
        "#;

        let settings = parse_planning_settings(visible);
        assert_eq!(
            settings.tempo_base_url.as_ref().map(Url::as_str),
            Some("https://school.mytempo.online/user/login")
        );
        assert!(settings.sundays_visible);

        let settings = parse_planning_settings(hidden);
        assert_eq!(settings.tempo_base_url, None);
        assert!(!settings.sundays_visible);

        let settings = parse_planning_settings("<html>no planning script</html>");
        assert_eq!(settings.tempo_base_url, None);
        assert!(!settings.sundays_visible);
    }

    #[test]
    fn converts_calendar_html_to_plain_text() {
        assert_eq!(
            html_to_text("Développement Web<br>Campus Lyon &amp; salle B204"),
            "Développement Web\nCampus Lyon & salle B204"
        );
        assert_eq!(
            html_to_text("Team Bulding (Cours)\r\n<br />\r\nDUCHEMIN Loïc <br />\r\nSalle Ada LOVELACE\r\n (Campus Nord) \r\n"),
            "Team Bulding (Cours)\nDUCHEMIN Loïc\nSalle Ada LOVELACE\n(Campus Nord)"
        );
    }
}
