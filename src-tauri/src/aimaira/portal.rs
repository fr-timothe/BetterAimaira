use std::collections::{HashMap, HashSet};

use futures_util::StreamExt;
use reqwest::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Client, Url,
};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::CommandError;

use super::{
    current_timestamp_millis, html_to_text, is_login_document, is_login_page, stable_hash_hex,
};

const MAX_DOCUMENT_BYTES: u64 = 25 * 1024 * 1024;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum PortalResource {
    Grades,
    Absences,
    Profile,
    Documents,
    Questionnaires,
}

impl PortalResource {
    fn route(self) -> &'static str {
        match self {
            Self::Grades => "Note",
            Self::Absences => "Absence",
            Self::Profile => "Profil",
            Self::Documents => "Document",
            Self::Questionnaires => "Questionnaire",
        }
    }

    fn unavailable_error(self) -> &'static str {
        match self {
            Self::Grades => "grades_unavailable",
            Self::Absences => "absences_unavailable",
            Self::Profile => "profile_unavailable",
            Self::Documents => "documents_unavailable",
            Self::Questionnaires => "questionnaires_unavailable",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalPage {
    pub resource: PortalResource,
    pub fetched_at: u64,
    pub title: String,
    pub headings: Vec<String>,
    pub tables: Vec<PortalTable>,
    pub fields: Vec<PortalField>,
    pub documents: Vec<PortalDocument>,
    /// Only filled for the grades page: the portal nests grades in year →
    /// block → course accordions that carry no `<table>` at all.
    pub grade_periods: Vec<GradePeriod>,
    /// Only filled for the absences page: same year → block accordions, each
    /// block holding one table of missed sessions.
    pub absence_periods: Vec<AbsencePeriod>,
    /// Only filled for the questionnaires page: one entry per portal response.
    pub questionnaires: Vec<QuestionnaireSummary>,
    pub markup_recognized: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireSummary {
    pub id: String,
    pub title: String,
    pub context: String,
    pub deadline: Option<String>,
    pub status: String,
    pub completed: bool,
    pub response_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireDetail {
    pub title: String,
    pub completed: bool,
    pub pages: Vec<QuestionnairePage>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnairePage {
    pub id: String,
    pub title: Option<String>,
    pub questions: Vec<QuestionnaireQuestion>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireQuestion {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub description: Option<String>,
    pub required: bool,
    pub options: Vec<QuestionnaireOption>,
    pub answers: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradePeriod {
    pub id: String,
    pub label: String,
    pub blocks: Vec<GradeBlock>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeBlock {
    pub id: String,
    pub label: String,
    pub bulletin_path: Option<String>,
    pub transcript_path: Option<String>,
    pub sections: Vec<GradeSection>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeSection {
    pub label: Option<String>,
    pub courses: Vec<GradeCourse>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeCourse {
    pub id: String,
    pub code: Option<String>,
    pub name: String,
    pub notebook_path: Option<String>,
    pub evaluations: Vec<GradeEvaluation>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeEvaluation {
    pub label: String,
    pub score: Option<String>,
    pub scale: Option<String>,
    /// Raw portal wording: `50,00%` for a weighting, `Pondération : 20,00` for
    /// a sub-evaluation. Kept verbatim because the two are not comparable.
    pub weight: Option<String>,
    pub children: Vec<GradeEvaluation>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsencePeriod {
    pub id: String,
    pub label: String,
    pub blocks: Vec<AbsenceBlock>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceBlock {
    pub id: String,
    pub label: String,
    pub report_path: Option<String>,
    pub entries: Vec<AbsenceEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceEntry {
    pub id: String,
    pub date: String,
    /// The portal packs the start time into the date cell; split so a list can
    /// group by day without re-parsing.
    pub time: Option<String>,
    pub course: String,
    /// Raw portal wording: `3,25` counts hours, `1h30` does not.
    pub duration: Option<String>,
    /// The portal names this column `Excusée`, and leaves it empty while the
    /// request is still being handled: `None` is pending, not refused.
    pub excused: Option<bool>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalTable {
    pub context: Vec<String>,
    pub caption: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalField {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PortalDocumentKind {
    AbsenceReport,
    GradeBulletin,
    GradeTranscript,
    EnrollmentCertificate,
    SchoolCertificate,
    SchoolTranscript,
    GradeReport,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalDocument {
    pub kind: PortalDocumentKind,
    pub label: String,
    pub request_path: String,
    pub suggested_filename: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Grade {
    pub id: String,
    pub subject: String,
    pub label: String,
    pub score: String,
    pub scale: Option<String>,
    pub coefficient: Option<String>,
    pub average: Option<String>,
}

pub async fn load_portal_resource(
    client: &Client,
    portal_url: &Url,
    resource: PortalResource,
) -> Result<PortalPage, CommandError> {
    let endpoint = portal_url
        .join(resource.route())
        .map_err(|_| CommandError::new("invalid_portal_url"))?;
    let response = client
        .get(endpoint)
        .send()
        .await
        .map_err(|_| CommandError::new(resource.unavailable_error()))?;

    if !response.status().is_success() {
        return Err(CommandError::new(resource.unavailable_error()));
    }

    let final_url = response.url().clone();
    let body = response
        .text()
        .await
        .map_err(|_| CommandError::new(resource.unavailable_error()))?;
    let document = Html::parse_document(&body);
    if is_login_page(&final_url, "") || is_login_document(&document) {
        return Err(CommandError::new("session_expired"));
    }

    let fetched_at = current_timestamp_millis()?;
    Ok(parse_portal_document(
        resource, portal_url, &document, fetched_at,
    ))
}

pub async fn load_questionnaire_detail(
    client: &Client,
    portal_url: &Url,
    response_path: &str,
) -> Result<QuestionnaireDetail, CommandError> {
    let endpoint = validate_questionnaire_url(portal_url, response_path)?;
    let response = client
        .get(endpoint)
        .send()
        .await
        .map_err(|_| CommandError::new("questionnaire_unavailable"))?;

    if !response.status().is_success() {
        return Err(CommandError::new("questionnaire_unavailable"));
    }

    let final_url = response.url().clone();
    let body = response
        .text()
        .await
        .map_err(|_| CommandError::new("questionnaire_unavailable"))?;
    let document = Html::parse_document(&body);
    if is_login_page(&final_url, "") || is_login_document(&document) {
        return Err(CommandError::new("session_expired"));
    }

    parse_questionnaire_detail(&document)
        .ok_or_else(|| CommandError::new("questionnaire_invalid_response"))
}

pub async fn download_portal_document(
    client: &Client,
    portal_url: &Url,
    request_path: &str,
) -> Result<Vec<u8>, CommandError> {
    let endpoint = validate_document_url(portal_url, request_path)?;
    let response = client
        .get(endpoint)
        .send()
        .await
        .map_err(|_| CommandError::new("document_unavailable"))?;

    if !response.status().is_success() {
        return Err(CommandError::new("document_unavailable"));
    }
    let final_url = response.url().clone();
    if is_login_page(&final_url, "") {
        return Err(CommandError::new("session_expired"));
    }

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let content_length = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    if content_length.is_some_and(|length| length > MAX_DOCUMENT_BYTES) {
        return Err(CommandError::new("document_too_large"));
    }

    let mut body =
        Vec::with_capacity(content_length.unwrap_or_default().min(MAX_DOCUMENT_BYTES) as usize);
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| CommandError::new("document_unavailable"))?;
        append_document_chunk(&mut body, &chunk, MAX_DOCUMENT_BYTES)?;
    }
    if !content_type.starts_with("application/pdf") || !body.starts_with(b"%PDF-") {
        if std::str::from_utf8(&body)
            .ok()
            .is_some_and(|html| is_login_page(&final_url, html))
        {
            return Err(CommandError::new("session_expired"));
        }
        return Err(CommandError::new("document_invalid_response"));
    }

    Ok(body)
}

fn append_document_chunk(
    body: &mut Vec<u8>,
    chunk: &[u8],
    maximum_bytes: u64,
) -> Result<(), CommandError> {
    if body.len().saturating_add(chunk.len()) as u64 > maximum_bytes {
        return Err(CommandError::new("document_too_large"));
    }
    body.extend_from_slice(chunk);
    Ok(())
}

#[cfg(test)]
fn parse_portal_page(
    resource: PortalResource,
    portal_url: &Url,
    html: &str,
    fetched_at: u64,
) -> PortalPage {
    let document = Html::parse_document(html);
    parse_portal_document(resource, portal_url, &document, fetched_at)
}

fn parse_portal_document(
    resource: PortalResource,
    portal_url: &Url,
    document: &Html,
    fetched_at: u64,
) -> PortalPage {
    let title = select_first_text(document, "main h1, main h2, h1, title").unwrap_or_default();
    let (headings, tables) = parse_semantic_tables(document);
    let fields = parse_fields(document);
    let documents = parse_documents(document, portal_url);
    let grade_periods = match resource {
        PortalResource::Grades => parse_grade_periods(document, portal_url),
        _ => Vec::new(),
    };
    let absence_periods = match resource {
        PortalResource::Absences => parse_absence_periods(document, portal_url),
        _ => Vec::new(),
    };
    let questionnaires = match resource {
        PortalResource::Questionnaires => parse_questionnaires(document, portal_url),
        _ => Vec::new(),
    };
    let questionnaire_markup = resource == PortalResource::Questionnaires
        && title.to_ascii_lowercase().contains("questionnaire");
    let markup_recognized = !tables.is_empty()
        || !fields.is_empty()
        || !documents.is_empty()
        || !grade_periods.is_empty()
        || !absence_periods.is_empty()
        || !questionnaires.is_empty()
        || questionnaire_markup;

    PortalPage {
        resource,
        fetched_at,
        title,
        headings,
        tables,
        fields,
        documents,
        grade_periods,
        absence_periods,
        questionnaires,
        markup_recognized,
    }
}

fn parse_questionnaires(document: &Html, portal_url: &Url) -> Vec<QuestionnaireSummary> {
    let response_selector = Selector::parse("a[href*='/Questionnaire/Reponse']").unwrap();
    let title_selector = Selector::parse("strong").unwrap();
    let context_selector = Selector::parse(".col-lg-6").unwrap();
    let deadline_selector = Selector::parse(".col-lg-3").unwrap();
    let status_selector = Selector::parse("label").unwrap();
    let mut questionnaires = Vec::new();
    let mut seen = HashSet::new();

    for anchor in document.select(&response_selector) {
        let Some(url) = same_origin_url(portal_url, anchor.value().attr("href")) else {
            continue;
        };
        if !is_questionnaire_response_url(&url) {
            continue;
        }
        let response_path = request_path(&url);
        if !seen.insert(response_path.clone()) {
            continue;
        }

        let Some(row) = anchor
            .ancestors()
            .filter_map(ElementRef::wrap)
            .find(|element| element.value().classes().any(|class| class == "row"))
        else {
            continue;
        };
        let title = row
            .select(&title_selector)
            .next()
            .map(element_text)
            .unwrap_or_default();
        let context = row
            .select(&context_selector)
            .next()
            .map(element_text)
            .unwrap_or_default()
            .strip_prefix(&title)
            .unwrap_or_default()
            .trim()
            .to_owned();
        let deadline = row
            .select(&deadline_selector)
            .next()
            .map(element_text)
            .map(|value| {
                value
                    .split_once(':')
                    .map(|(_, deadline)| deadline.trim().to_owned())
                    .unwrap_or(value)
            })
            .filter(|value| !value.is_empty());
        let status = row
            .select(&status_selector)
            .next()
            .map(element_text)
            .unwrap_or_default();
        let normalized_status = normalize_header(&status);
        let action = normalize_header(&element_text(anchor));
        let completed = normalized_status.contains("finalis")
            || normalized_status.contains("complet")
            || normalized_status.contains("termin")
            || action.contains("consulter")
            || action.contains("view");

        questionnaires.push(QuestionnaireSummary {
            id: stable_hash_hex(&[&response_path]),
            title,
            context,
            deadline,
            status,
            completed,
            response_path,
        });
    }

    questionnaires
}

fn parse_questionnaire_detail(document: &Html) -> Option<QuestionnaireDetail> {
    let survey = extract_script_json(document, "jsonSurvey")?;
    let responses = extract_script_json(document, "jsonReponse").unwrap_or(Value::Null);
    let survey_object = survey.as_object()?;
    let title = survey_object
        .get("title")
        .and_then(Value::as_str)
        .map(html_to_text)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Questionnaire".to_owned());
    let pages = survey_object
        .get("pages")
        .and_then(Value::as_array)?
        .iter()
        .enumerate()
        .filter_map(|(index, page)| parse_questionnaire_page(page, &responses, index))
        .collect::<Vec<_>>();
    if pages.is_empty() {
        return None;
    }

    Some(QuestionnaireDetail {
        title,
        completed: questionnaire_is_complete(document),
        pages,
    })
}

fn parse_questionnaire_page(
    page: &Value,
    responses: &Value,
    page_index: usize,
) -> Option<QuestionnairePage> {
    let page = page.as_object()?;
    let page_name = page
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let id = if page_name.is_empty() {
        stable_hash_hex(&["questionnaire-page", &page_index.to_string()])
    } else {
        stable_hash_hex(&["questionnaire-page", page_name])
    };
    let title = page
        .get("title")
        .and_then(Value::as_str)
        .map(html_to_text)
        .filter(|value| !value.is_empty());
    let mut questions = Vec::new();
    if let Some(elements) = page.get("elements").and_then(Value::as_array) {
        collect_questionnaire_questions(elements, responses, &mut questions);
    }

    Some(QuestionnairePage {
        id,
        title,
        questions,
    })
}

fn collect_questionnaire_questions(
    elements: &[Value],
    responses: &Value,
    questions: &mut Vec<QuestionnaireQuestion>,
) {
    for element in elements {
        let Some(element) = element.as_object() else {
            continue;
        };
        if let Some(children) = element.get("elements").and_then(Value::as_array) {
            collect_questionnaire_questions(children, responses, questions);
        }

        let name = element
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let title = element
            .get("title")
            .and_then(Value::as_str)
            .map(html_to_text)
            .unwrap_or_default();
        if name.is_empty() || title.is_empty() {
            continue;
        }

        let kind = element
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        let options = questionnaire_options(element, &kind);
        let answer = responses.as_object().and_then(|values| values.get(name));
        let answers = answer
            .map(|answer| questionnaire_answers(answer, &options))
            .unwrap_or_default();
        questions.push(QuestionnaireQuestion {
            id: stable_hash_hex(&["questionnaire-question", name]),
            kind,
            title,
            description: element
                .get("description")
                .and_then(Value::as_str)
                .map(html_to_text)
                .filter(|value| !value.is_empty()),
            required: element
                .get("isRequired")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            options,
            answers,
        });
    }
}

fn questionnaire_options(
    element: &serde_json::Map<String, Value>,
    kind: &str,
) -> Vec<QuestionnaireOption> {
    if let Some(choices) = element.get("choices").and_then(Value::as_array) {
        return choices.iter().filter_map(questionnaire_option).collect();
    }
    if kind != "rating" {
        return Vec::new();
    }

    let minimum = element.get("rateMin").and_then(Value::as_i64).unwrap_or(1);
    let maximum = element.get("rateMax").and_then(Value::as_i64).unwrap_or(5);
    let step = element
        .get("rateStep")
        .and_then(Value::as_i64)
        .filter(|step| *step > 0)
        .unwrap_or(1);
    if minimum > maximum || maximum.saturating_sub(minimum) > 20 {
        return Vec::new();
    }

    (minimum..=maximum)
        .step_by(step as usize)
        .map(|value| QuestionnaireOption {
            value: value.to_string(),
            label: value.to_string(),
        })
        .collect()
}

fn questionnaire_option(choice: &Value) -> Option<QuestionnaireOption> {
    match choice {
        Value::String(value) => Some(QuestionnaireOption {
            value: value.clone(),
            label: value.clone(),
        }),
        Value::Number(value) => Some(QuestionnaireOption {
            value: value.to_string(),
            label: value.to_string(),
        }),
        Value::Object(choice) => {
            let value = choice.get("value").and_then(json_scalar_text)?;
            let label = choice
                .get("text")
                .and_then(json_scalar_text)
                .unwrap_or_else(|| value.clone());
            Some(QuestionnaireOption { value, label })
        }
        _ => None,
    }
}

fn questionnaire_answers(answer: &Value, options: &[QuestionnaireOption]) -> Vec<String> {
    match answer {
        Value::Array(values) => values
            .iter()
            .filter_map(json_scalar_text)
            .map(|value| questionnaire_answer_label(&value, options))
            .collect(),
        Value::Object(values) => values
            .iter()
            .filter_map(|(key, value)| {
                json_scalar_text(value).map(|value| {
                    format!("{}: {}", html_to_text(key), questionnaire_answer_label(&value, options))
                })
            })
            .collect(),
        value => json_scalar_text(value)
            .map(|value| vec![questionnaire_answer_label(&value, options)])
            .unwrap_or_default(),
    }
}

fn questionnaire_answer_label(value: &str, options: &[QuestionnaireOption]) -> String {
    options
        .iter()
        .find(|option| option.value == value)
        .map(|option| option.label.clone())
        .unwrap_or_else(|| html_to_text(value))
}

fn json_scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(html_to_text(value)),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn extract_script_json(document: &Html, variable: &str) -> Option<Value> {
    let script_selector = Selector::parse("script").unwrap();
    let declarations = [
        format!("var {variable}"),
        format!("let {variable}"),
        format!("const {variable}"),
    ];

    for script in document.select(&script_selector) {
        let source = script.text().collect::<String>();
        let Some(declaration_start) = declarations
            .iter()
            .filter_map(|declaration| source.find(declaration))
            .min()
        else {
            continue;
        };
        let assignment = source[declaration_start..].find('=')? + declaration_start + 1;
        let value_source = source[assignment..].trim_start();
        let mut values = serde_json::Deserializer::from_str(value_source).into_iter::<Value>();
        if let Some(Ok(value)) = values.next() {
            return Some(value);
        }
    }
    None
}

fn questionnaire_is_complete(document: &Html) -> bool {
    let script_selector = Selector::parse("script").unwrap();
    document.select(&script_selector).any(|script| {
        let source = script.text().collect::<String>();
        source.contains("isComplete: true") || source.contains("isComplete:true")
    })
}

fn validate_questionnaire_url(
    portal_url: &Url,
    response_path: &str,
) -> Result<Url, CommandError> {
    if !response_path.starts_with('/')
        || response_path.starts_with("//")
        || response_path.contains('#')
    {
        return Err(CommandError::new("invalid_questionnaire_request"));
    }
    let url = portal_url
        .join(response_path)
        .map_err(|_| CommandError::new("invalid_questionnaire_request"))?;
    if url.origin() != portal_url.origin() || !is_questionnaire_response_url(&url) {
        return Err(CommandError::new("invalid_questionnaire_request"));
    }
    Ok(url)
}

fn is_questionnaire_response_url(url: &Url) -> bool {
    url.path().trim_end_matches('/').eq_ignore_ascii_case("/Questionnaire/Reponse")
        && url
            .query_pairs()
            .any(|(key, value)| key.eq_ignore_ascii_case("idReponse") && !value.is_empty())
}

/// The grades page ships no table: years, blocks and courses are Bootstrap
/// accordion panels, and each course lists its evaluations in a `<dl>`.
fn parse_grade_periods(document: &Html, portal_url: &Url) -> Vec<GradePeriod> {
    let period_selector = Selector::parse("#accordion-periode > .panel").unwrap();
    let block_selector = Selector::parse(".panel-group > .panel").unwrap();
    let heading_selector = Selector::parse(".panel-heading").unwrap();
    let title_selector = Selector::parse(".panel-title").unwrap();
    let anchor_selector = Selector::parse("a[href]").unwrap();
    let tiles_selector = Selector::parse(".tiles-list").unwrap();
    let tile_selector = Selector::parse(".tile").unwrap();
    let section_title_selector = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();
    let mut periods = Vec::new();

    for period in document.select(&period_selector) {
        let label = period
            .select(&heading_selector)
            .next()
            .and_then(|heading| heading.select(&title_selector).next())
            .map(element_text)
            .unwrap_or_default();
        let mut blocks = Vec::new();

        for block in period.select(&block_selector) {
            let heading = block.select(&heading_selector).next();
            let block_label = heading
                .and_then(|heading| heading.select(&title_selector).next())
                .map(element_text)
                .unwrap_or_default();
            let mut bulletin_path = None;
            let mut transcript_path = None;
            if let Some(heading) = heading {
                for anchor in heading.select(&anchor_selector) {
                    let Some(url) = same_origin_url(portal_url, anchor.value().attr("href")) else {
                        continue;
                    };
                    match document_kind(url.path()) {
                        Some(PortalDocumentKind::GradeBulletin) => {
                            bulletin_path.get_or_insert_with(|| request_path(&url));
                        }
                        Some(PortalDocumentKind::GradeTranscript) => {
                            transcript_path.get_or_insert_with(|| request_path(&url));
                        }
                        _ => {}
                    }
                }
            }

            let block_id = stable_hash_hex(&[&label, &block_label]);
            let mut sections = Vec::<GradeSection>::new();
            if let Some(tiles) = block.select(&tiles_selector).next() {
                for child in tiles.children().filter_map(ElementRef::wrap) {
                    if let Some(tile) = child.select(&tile_selector).next() {
                        let course = parse_grade_course(tile, portal_url, &block_id);
                        match sections.last_mut() {
                            Some(section) => section.courses.push(course),
                            None => sections.push(GradeSection {
                                label: None,
                                courses: vec![course],
                            }),
                        }
                        continue;
                    }

                    // A bare column between two tile rows carries the heading of
                    // the learning track the tiles below it belong to.
                    let section_label = child
                        .select(&section_title_selector)
                        .map(element_text)
                        .find(|text| !text.is_empty());
                    if let Some(section_label) = section_label {
                        sections.push(GradeSection {
                            label: Some(section_label),
                            courses: Vec::new(),
                        });
                    }
                }
            }
            sections.retain(|section| !section.courses.is_empty());

            if block_label.is_empty() && sections.is_empty() {
                continue;
            }
            blocks.push(GradeBlock {
                id: block_id,
                label: block_label,
                bulletin_path,
                transcript_path,
                sections,
            });
        }

        if label.is_empty() && blocks.is_empty() {
            continue;
        }
        periods.push(GradePeriod {
            id: stable_hash_hex(&[&label]),
            label,
            blocks,
        });
    }

    periods
}

fn parse_grade_course(tile: ElementRef<'_>, portal_url: &Url, block_id: &str) -> GradeCourse {
    let strong_selector = Selector::parse(".panel-heading strong").unwrap();
    let anchor_selector = Selector::parse(".panel-heading a[href]").unwrap();
    let item_selector = Selector::parse("dt, dd").unwrap();
    let small_selector = Selector::parse("small").unwrap();

    let title = tile
        .select(&strong_selector)
        .next()
        .map(element_text)
        .unwrap_or_default();
    let (code, name) = match title.split_once(" - ") {
        Some((code, name)) if !code.is_empty() && !name.is_empty() => {
            (Some(code.to_owned()), name.to_owned())
        }
        _ => (None, title.clone()),
    };
    let notebook_path = tile
        .select(&anchor_selector)
        .filter_map(|anchor| same_origin_url(portal_url, anchor.value().attr("href")))
        .find(|url| {
            url.path()
                .to_ascii_lowercase()
                .starts_with("/saisiecahiertexte")
        })
        .map(|url| request_path(&url));

    let mut evaluations = Vec::<GradeEvaluation>::new();
    let mut pending_score: Option<(bool, Option<String>, Option<String>)> = None;
    for item in tile.select(&item_selector) {
        match item.value().name() {
            "dt" => {
                let text = element_text(item);
                // A leading bullet marks a sub-evaluation of the entry above it.
                let is_child = text.starts_with('•');
                let raw_score = text.trim_start_matches('•').trim();
                let (score, scale) = if raw_score.is_empty() {
                    (None, None)
                } else {
                    let (score, scale) = split_score(raw_score);
                    (Some(score), scale)
                };
                pending_score = Some((is_child, score, scale));
            }
            "dd" => {
                let Some((is_child, score, scale)) = pending_score.take() else {
                    continue;
                };
                let full_text = element_text(item);
                let weight = item.select(&small_selector).next().map(element_text);
                let label = match weight.as_deref() {
                    Some(weight) => full_text.trim_end_matches(weight).trim().to_owned(),
                    None => full_text,
                };
                let weight = weight.map(|weight| {
                    weight
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .trim()
                        .to_owned()
                });
                if label.is_empty() && score.is_none() {
                    continue;
                }

                let evaluation = GradeEvaluation {
                    label,
                    score,
                    scale,
                    weight,
                    children: Vec::new(),
                };
                match evaluations.last_mut() {
                    Some(parent) if is_child => parent.children.push(evaluation),
                    _ => evaluations.push(evaluation),
                }
            }
            _ => {}
        }
    }

    GradeCourse {
        id: stable_hash_hex(&[block_id, code.as_deref().unwrap_or(&name)]),
        code,
        name,
        notebook_path,
        evaluations,
    }
}

/// The absences page reuses the grades layout: year → block Bootstrap
/// accordions, each block carrying its own attendance report link and a single
/// table of missed sessions.
fn parse_absence_periods(document: &Html, portal_url: &Url) -> Vec<AbsencePeriod> {
    let period_selector = Selector::parse("#accordion-periode > .panel").unwrap();
    let block_selector = Selector::parse(".panel-group > .panel").unwrap();
    let heading_selector = Selector::parse(".panel-heading").unwrap();
    let title_selector = Selector::parse(".panel-title").unwrap();
    let anchor_selector = Selector::parse("a[href]").unwrap();
    let table_selector = Selector::parse("table").unwrap();
    let header_selector = Selector::parse("thead th").unwrap();
    let row_selector = Selector::parse("tbody tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    let mut periods = Vec::new();

    for period in document.select(&period_selector) {
        let label = period
            .select(&heading_selector)
            .next()
            .and_then(|heading| heading.select(&title_selector).next())
            .map(element_text)
            .unwrap_or_default();
        let mut blocks = Vec::new();

        for block in period.select(&block_selector) {
            let block_label = block
                .select(&heading_selector)
                .next()
                .and_then(|heading| heading.select(&title_selector).next())
                .map(element_text)
                .unwrap_or_default();
            // The report link sits in the block body, above the table.
            let report_path = block
                .select(&anchor_selector)
                .filter_map(|anchor| same_origin_url(portal_url, anchor.value().attr("href")))
                .find(|url| document_kind(url.path()) == Some(PortalDocumentKind::AbsenceReport))
                .map(|url| request_path(&url));

            let block_id = stable_hash_hex(&[&label, &block_label]);
            let mut entries = Vec::new();

            if let Some(table) = block.select(&table_selector).next() {
                let headers = table
                    .select(&header_selector)
                    .map(element_text)
                    .collect::<Vec<_>>();
                let date_column = find_header(&headers, &["date", "seance", "jour"]);
                let course_column = find_header(&headers, &["cours", "matiere", "libelle"]);
                let duration_column = find_header(&headers, &["duree", "heure", "volume"]);
                let excused_column =
                    find_header(&headers, &["excuse", "justifi", "statut", "etat"]);
                let reason_column = find_header(&headers, &["motif", "raison", "commentaire"]);

                for (row_index, row) in table.select(&row_selector).enumerate() {
                    let cells = row.select(&cell_selector).map(element_text).collect::<Vec<_>>();
                    if cells.iter().all(String::is_empty) {
                        continue;
                    }

                    let cell = |column: Option<usize>| {
                        column
                            .and_then(|index| cells.get(index))
                            .map(String::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                    };
                    let (date, time) = split_date_time(cell(date_column).unwrap_or_default());

                    entries.push(AbsenceEntry {
                        id: stable_hash_hex(&[&block_id, &row_index.to_string()]),
                        date,
                        time,
                        course: cell(course_column).unwrap_or_default().to_owned(),
                        duration: cell(duration_column).map(str::to_owned),
                        excused: cell(excused_column).and_then(parse_excused),
                        reason: cell(reason_column).map(str::to_owned),
                    });
                }
            }

            if block_label.is_empty() && entries.is_empty() && report_path.is_none() {
                continue;
            }
            blocks.push(AbsenceBlock {
                id: block_id,
                label: block_label,
                report_path,
                entries,
            });
        }

        if label.is_empty() && blocks.is_empty() {
            continue;
        }
        periods.push(AbsencePeriod {
            id: stable_hash_hex(&[&label]),
            label,
            blocks,
        });
    }

    periods
}

/// `29/09/2025 08:45` is one cell on the portal. The time is optional: some
/// rows only name the day.
fn split_date_time(cell: &str) -> (String, Option<String>) {
    let mut date_parts = Vec::new();
    let mut time = None;

    for part in cell.split_whitespace() {
        let is_time = time.is_none()
            && part.contains(':')
            && part
                .chars()
                .all(|character| character.is_ascii_digit() || character == ':');
        if is_time {
            time = Some(part.to_owned());
        } else {
            date_parts.push(part);
        }
    }

    (date_parts.join(" "), time)
}

/// The `Excusée` column answers `Oui`/`Non`, and stays empty while the school
/// has not ruled on the justification yet.
fn parse_excused(value: &str) -> Option<bool> {
    let normalized = normalize_header(value);
    if normalized.starts_with("non") || normalized == "n" || normalized == "false" {
        return Some(false);
    }
    if normalized.starts_with("oui")
        || normalized.starts_with("excuse")
        || normalized.starts_with("justifi")
        || normalized.starts_with("valide")
        || normalized.starts_with("accepte")
        || normalized == "o"
        || normalized == "x"
        || normalized == "true"
        || normalized == "yes"
    {
        return Some(true);
    }
    None
}

fn same_origin_url(portal_url: &Url, href: Option<&str>) -> Option<Url> {
    let url = portal_url.join(href?).ok()?;
    (url.origin() == portal_url.origin()).then_some(url)
}

pub fn extract_grades(page: &PortalPage) -> Vec<Grade> {
    let mut grades = Vec::new();
    let mut seen = HashSet::new();

    for period in &page.grade_periods {
        collect_period_grades(period, &mut grades, &mut seen);
    }

    for table in &page.tables {
        let Some(score_index) =
            find_header(&table.headers, &["note", "résultat", "resultat", "score"])
        else {
            continue;
        };
        let label_index = find_header(
            &table.headers,
            &[
                "libellé",
                "libelle",
                "évaluation",
                "evaluation",
                "intitulé",
                "intitule",
                "devoir",
            ],
        );
        let coefficient_index = find_header(&table.headers, &["coefficient", "coeff"]);
        let average_index = find_header(&table.headers, &["moyenne"]);
        let subject = table
            .context
            .last()
            .or(table.caption.as_ref())
            .cloned()
            .unwrap_or_else(|| "Note".to_owned());

        for row in &table.rows {
            let Some(score) = row
                .get(score_index)
                .map(String::as_str)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };
            let label = label_index
                .and_then(|index| row.get(index))
                .filter(|value| !value.is_empty())
                .cloned()
                .unwrap_or_else(|| subject.clone());
            let (score, scale) = split_score(score);
            let coefficient = coefficient_index
                .and_then(|index| row.get(index))
                .filter(|value| !value.is_empty())
                .cloned();
            let average = average_index
                .and_then(|index| row.get(index))
                .filter(|value| !value.is_empty())
                .cloned();
            let id = stable_hash_hex(&[
                &subject,
                &label,
                scale.as_deref().unwrap_or_default(),
                coefficient.as_deref().unwrap_or_default(),
            ]);
            if seen.insert(id.clone()) {
                grades.push(Grade {
                    id,
                    subject: subject.clone(),
                    label,
                    score,
                    scale,
                    coefficient,
                    average,
                });
            }
        }
    }

    grades
}

/// Extract only the grades from the most recent school year for the home page.
/// The full list is still synchronized separately so existing grade alerts keep
/// their history across school-year changes.
pub fn extract_latest_grades(page: &PortalPage) -> Vec<Grade> {
    let Some(latest_period) = page
        .grade_periods
        .iter()
        .filter(|period| period_start_year(&period.label).is_some())
        .max_by_key(|period| period_start_year(&period.label).unwrap_or_default())
    else {
        return extract_grades(page);
    };

    let mut grades = Vec::new();
    let mut seen = HashSet::new();
    collect_period_grades(latest_period, &mut grades, &mut seen);
    grades
}

fn collect_period_grades(period: &GradePeriod, grades: &mut Vec<Grade>, seen: &mut HashSet<String>) {
    for block in &period.blocks {
        for section in &block.sections {
            for course in &section.courses {
                collect_course_grades(course, grades, seen);
            }
        }
    }
}

fn period_start_year(label: &str) -> Option<u32> {
    label
        .split(|character: char| !character.is_ascii_digit())
        .find_map(|part| (part.len() == 4).then(|| part.parse().ok()).flatten())
}

/// Sub-evaluations are graded work too: a new one has to raise an alert like
/// any other, so the flat list keeps them next to their parent.
fn collect_course_grades(
    course: &GradeCourse,
    grades: &mut Vec<Grade>,
    seen: &mut HashSet<String>,
) {
    fn push(
        course: &GradeCourse,
        evaluation: &GradeEvaluation,
        grades: &mut Vec<Grade>,
        seen: &mut HashSet<String>,
    ) {
        if let Some(score) = evaluation.score.as_ref().filter(|score| !score.is_empty()) {
            let id = stable_hash_hex(&[
                &course.id,
                &evaluation.label,
                score,
                evaluation.weight.as_deref().unwrap_or_default(),
            ]);
            if seen.insert(id.clone()) {
                grades.push(Grade {
                    id,
                    subject: course.name.clone(),
                    label: evaluation.label.clone(),
                    score: score.clone(),
                    scale: evaluation.scale.clone(),
                    coefficient: evaluation.weight.clone(),
                    average: None,
                });
            }
        }
        for child in &evaluation.children {
            push(course, child, grades, seen);
        }
    }

    for evaluation in &course.evaluations {
        push(course, evaluation, grades, seen);
    }
}

fn find_header(headers: &[String], candidates: &[&str]) -> Option<usize> {
    headers.iter().position(|header| {
        let normalized = normalize_header(header);
        candidates
            .iter()
            .any(|candidate| normalized.contains(candidate))
    })
}

fn normalize_header(value: &str) -> String {
    value
        .to_lowercase()
        .replace([' ', '-', '_', '.'], "")
        .replace(['é', 'è', 'ê'], "e")
}

fn split_score(value: &str) -> (String, Option<String>) {
    let Some((score, scale)) = value.split_once('/') else {
        return (value.trim().to_owned(), None);
    };
    let score = score.trim().to_owned();
    let scale = scale.trim().to_owned();
    (score, (!scale.is_empty()).then_some(scale))
}

fn parse_semantic_tables(document: &Html) -> (Vec<String>, Vec<PortalTable>) {
    let semantic_selector = Selector::parse("h1, h2, h3, h4, h5, h6, table").unwrap();
    let mut context = Vec::<String>::new();
    let mut headings = Vec::new();
    let mut tables = Vec::new();

    for element in document.select(&semantic_selector) {
        let name = element.value().name();
        if let Some(level) = heading_level(name) {
            let text = element_text(element);
            if text.is_empty() {
                continue;
            }
            context.truncate(level.saturating_sub(1));
            while context.len() < level.saturating_sub(1) {
                context.push(String::new());
            }
            context.push(text.clone());
            headings.push(text);
            continue;
        }

        if let Some(table) = parse_table(element, &context) {
            tables.push(table);
        }
    }

    (headings, tables)
}

fn parse_table(table: ElementRef<'_>, context: &[String]) -> Option<PortalTable> {
    let caption = table
        .select(&Selector::parse("caption").unwrap())
        .map(element_text)
        .find(|text| !text.is_empty());
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("th, td").unwrap();
    let data_cell_selector = Selector::parse("td").unwrap();
    let mut headers = Vec::new();
    let mut rows = Vec::new();

    for row in table.select(&row_selector) {
        let cells = row
            .select(&cell_selector)
            .map(element_text)
            .collect::<Vec<_>>();
        if cells.iter().all(String::is_empty) {
            continue;
        }

        let is_header_row = row.select(&data_cell_selector).next().is_none();
        if headers.is_empty() && is_header_row {
            headers = cells;
        } else {
            rows.push(cells);
        }
    }

    if headers.is_empty() && rows.is_empty() {
        return None;
    }

    Some(PortalTable {
        context: context
            .iter()
            .filter(|heading| !heading.is_empty())
            .cloned()
            .collect(),
        caption,
        headers,
        rows,
    })
}

fn parse_fields(document: &Html) -> Vec<PortalField> {
    let mut fields = Vec::new();
    let mut seen = HashSet::new();
    let definition_selector = Selector::parse("dl").unwrap();
    let definition_item_selector = Selector::parse("dt, dd").unwrap();

    for list in document.select(&definition_selector) {
        let mut label = None;
        for item in list.select(&definition_item_selector) {
            match item.value().name() {
                "dt" => label = Some(element_text(item)),
                "dd" => {
                    if let Some(label) = label.take() {
                        push_field(&mut fields, &mut seen, label, element_text(item));
                    }
                }
                _ => {}
            }
        }
    }

    let controls = collect_form_controls(document);
    let label_selector = Selector::parse("label[for]").unwrap();
    for label in document.select(&label_selector) {
        let Some(control_id) = label.value().attr("for") else {
            continue;
        };
        let Some(value) = controls.get(control_id) else {
            continue;
        };
        push_field(&mut fields, &mut seen, element_text(label), value.clone());
    }

    fields
}

fn collect_form_controls(document: &Html) -> HashMap<String, String> {
    let selector = Selector::parse("input[id], select[id], textarea[id]").unwrap();
    let selected_option_selector = Selector::parse("option[selected]").unwrap();
    let option_selector = Selector::parse("option").unwrap();
    let mut controls = HashMap::new();

    for control in document.select(&selector) {
        let Some(id) = control.value().attr("id") else {
            continue;
        };
        let value = match control.value().name() {
            "input" => {
                let input_type = control.value().attr("type").unwrap_or("text");
                if matches!(
                    input_type,
                    "hidden"
                        | "password"
                        | "file"
                        | "checkbox"
                        | "radio"
                        | "button"
                        | "submit"
                        | "reset"
                        | "image"
                ) {
                    continue;
                }
                control.value().attr("value").unwrap_or_default().to_owned()
            }
            "select" => control
                .select(&selected_option_selector)
                .next()
                .or_else(|| control.select(&option_selector).next())
                .map(element_text)
                .unwrap_or_default(),
            "textarea" => element_text(control),
            _ => continue,
        };

        controls.insert(id.to_owned(), html_to_text(&value));
    }

    controls
}

fn push_field(
    fields: &mut Vec<PortalField>,
    seen: &mut HashSet<(String, String)>,
    label: String,
    value: String,
) {
    let label = html_to_text(&label);
    let value = html_to_text(&value);
    if label.is_empty() || value.is_empty() || !seen.insert((label.clone(), value.clone())) {
        return;
    }
    fields.push(PortalField { label, value });
}

fn parse_documents(document: &Html, portal_url: &Url) -> Vec<PortalDocument> {
    let anchor_selector = Selector::parse("a[href]").unwrap();
    let mut seen = HashSet::new();
    let mut documents = Vec::new();

    for anchor in document.select(&anchor_selector) {
        let Some(href) = anchor.value().attr("href") else {
            continue;
        };
        let Ok(url) = portal_url.join(href) else {
            continue;
        };
        if url.origin() != portal_url.origin() {
            continue;
        }
        let Some(kind) = document_kind(url.path()) else {
            continue;
        };
        let request_path = request_path(&url);
        if !seen.insert(request_path.clone()) {
            continue;
        }
        let label = element_text(anchor);
        let label = if label.is_empty() {
            anchor
                .value()
                .attr("title")
                .unwrap_or("Document")
                .to_owned()
        } else {
            label
        };
        documents.push(PortalDocument {
            kind,
            label,
            request_path,
            suggested_filename: anchor.value().attr("download").map(str::to_owned),
        });
    }

    documents
}

fn validate_document_url(portal_url: &Url, request_path: &str) -> Result<Url, CommandError> {
    if !request_path.starts_with('/')
        || request_path.starts_with("//")
        || request_path.contains('#')
    {
        return Err(CommandError::new("invalid_document_request"));
    }
    let url = portal_url
        .join(request_path)
        .map_err(|_| CommandError::new("invalid_document_request"))?;
    if url.origin() != portal_url.origin() || document_kind(url.path()).is_none() {
        return Err(CommandError::new("invalid_document_request"));
    }
    Ok(url)
}

fn document_kind(path: &str) -> Option<PortalDocumentKind> {
    match path.trim_end_matches('/').to_ascii_lowercase().as_str() {
        "/absence/downloadreleveabsence" => Some(PortalDocumentKind::AbsenceReport),
        "/note/downloadbulletin" => Some(PortalDocumentKind::GradeBulletin),
        "/note/downloadreleve" => Some(PortalDocumentKind::GradeTranscript),
        "/document/downloadcertificatreinscription" => {
            Some(PortalDocumentKind::EnrollmentCertificate)
        }
        "/document/downloadcertificatscolarite" => Some(PortalDocumentKind::SchoolCertificate),
        "/document/downloadrelevescolarite" => Some(PortalDocumentKind::SchoolTranscript),
        "/document/downloadbulletinnoteid" => Some(PortalDocumentKind::GradeReport),
        _ => None,
    }
}

fn request_path(url: &Url) -> String {
    match url.query() {
        Some(query) => format!("{}?{query}", url.path()),
        None => url.path().to_owned(),
    }
}

fn heading_level(name: &str) -> Option<usize> {
    name.strip_prefix('h')?
        .parse::<usize>()
        .ok()
        .filter(|level| (1..=6).contains(level))
}

fn select_first_text(document: &Html, selector: &str) -> Option<String> {
    document
        .select(&Selector::parse(selector).ok()?)
        .map(element_text)
        .find(|text| !text.is_empty())
}

fn element_text(element: ElementRef<'_>) -> String {
    element
        .text()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{
        append_document_chunk, extract_grades, extract_latest_grades, parse_portal_page,
        parse_questionnaire_detail,
        validate_document_url, validate_questionnaire_url, PortalDocumentKind, PortalResource,
    };
    use reqwest::Url;
    use scraper::Html;

    #[test]
    fn parses_semantic_tables_fields_and_safe_documents() {
        let portal = Url::parse("https://school.example/").unwrap();
        let html = r#"
            <main>
                <h1>Mes notes</h1>
                <h2>2025 / 2026</h2>
                <h3>Développement Web</h3>
                <table>
                    <caption>Évaluations</caption>
                    <thead><tr><th>Libellé</th><th>Note</th></tr></thead>
                    <tbody><tr><td>Projet &amp; oral</td><td>15 / 20</td></tr></tbody>
                </table>
                <dl><dt>Campus</dt><dd>Lyon</dd></dl>
                <label for="email">E-mail</label><input id="email" value="student@example.com">
                <input id="csrf" type="hidden" value="secret">
                <a href="/Note/DownloadBulletin?IdInscriptionSequence=opaque">Bulletin</a>
                <a href="https://evil.example/Note/DownloadBulletin?id=1">External</a>
                <a href="/Paiement/DownloadFacture?id=1">Invoice</a>
            </main>
        "#;

        let page = parse_portal_page(PortalResource::Grades, &portal, html, 1_777_777_777_777);

        assert_eq!(page.title, "Mes notes");
        assert_eq!(page.fetched_at, 1_777_777_777_777);
        assert_eq!(page.tables.len(), 1);
        assert_eq!(
            page.tables[0].context,
            ["Mes notes", "2025 / 2026", "Développement Web"]
        );
        assert_eq!(page.tables[0].headers, ["Libellé", "Note"]);
        assert_eq!(page.tables[0].rows, [["Projet & oral", "15 / 20"]]);
        assert_eq!(page.fields.len(), 2);
        assert_eq!(page.documents.len(), 1);
        assert_eq!(page.documents[0].kind, PortalDocumentKind::GradeBulletin);
        assert_eq!(
            page.documents[0].request_path,
            "/Note/DownloadBulletin?IdInscriptionSequence=opaque"
        );
        assert!(page.markup_recognized);
    }

    #[test]
    fn parses_questionnaire_list_entries_without_exposing_cross_origin_links() {
        let portal = Url::parse("https://school.example/").unwrap();
        let html = r#"
            <main>
                <h1>Liste des Questionnaires</h1>
                <div class="row">
                    <div class="col-lg-6"><strong>Course survey</strong><br>Web development - Teacher</div>
                    <div class="col-lg-3"><strong>Deadline:</strong> 31/08/2026</div>
                    <div class="col-lg-2">
                        <label>Completed</label><br>
                        <a href="/Questionnaire/Reponse?idReponse=opaque-response">View</a>
                    </div>
                </div>
                <a href="https://evil.example/Questionnaire/Reponse?idReponse=stolen">External</a>
            </main>
        "#;

        let page = parse_portal_page(PortalResource::Questionnaires, &portal, html, 1);

        assert!(page.markup_recognized);
        assert_eq!(page.questionnaires.len(), 1);
        let questionnaire = &page.questionnaires[0];
        assert_eq!(questionnaire.title, "Course survey");
        assert_eq!(questionnaire.context, "Web development - Teacher");
        assert_eq!(questionnaire.deadline.as_deref(), Some("31/08/2026"));
        assert_eq!(questionnaire.status, "Completed");
        assert!(questionnaire.completed);
        assert_eq!(
            questionnaire.response_path,
            "/Questionnaire/Reponse?idReponse=opaque-response"
        );
    }

    #[test]
    fn parses_survey_json_and_saved_answers_from_questionnaire_detail() {
        let html = r#"
            <main><h1>Course survey</h1></main>
            <script>
                var jsonSurvey = {
                    "title": "Course survey",
                    "pages": [{
                        "name": "page1",
                        "title": "Teaching",
                        "elements": [
                            {
                                "type": "rating",
                                "name": "quality",
                                "title": "How was the course?",
                                "description": "1 is low, 5 is high",
                                "isRequired": true
                            },
                            {
                                "type": "checkbox",
                                "name": "strengths",
                                "title": "What worked?",
                                "choices": [
                                    { "value": "pace", "text": "Pace" },
                                    { "value": "examples", "text": "Examples" }
                                ]
                            }
                        ]
                    }]
                };
                var jsonReponse = { "quality": 4, "strengths": ["examples"] };
                new SurveyJsHelper({ isComplete: true });
            </script>
        "#;
        let document = Html::parse_document(html);

        let detail = parse_questionnaire_detail(&document).unwrap();

        assert_eq!(detail.title, "Course survey");
        assert!(detail.completed);
        assert_eq!(detail.pages.len(), 1);
        assert_eq!(detail.pages[0].title.as_deref(), Some("Teaching"));
        assert_eq!(detail.pages[0].questions.len(), 2);
        let rating = &detail.pages[0].questions[0];
        assert_eq!(rating.kind, "rating");
        assert_eq!(rating.options.len(), 5);
        assert_eq!(rating.answers, ["4"]);
        let checkbox = &detail.pages[0].questions[1];
        assert_eq!(checkbox.options[1].label, "Examples");
        assert_eq!(checkbox.answers, ["Examples"]);
    }

    #[test]
    fn questionnaire_details_are_restricted_to_the_read_only_response_route() {
        let portal = Url::parse("https://school.example/").unwrap();

        assert!(validate_questionnaire_url(
            &portal,
            "/Questionnaire/Reponse?idReponse=opaque-response"
        )
        .is_ok());
        for invalid in [
            "https://evil.example/Questionnaire/Reponse?idReponse=stolen",
            "//evil.example/Questionnaire/Reponse?idReponse=stolen",
            "/Questionnaire/Reponse",
            "/Questionnaire/Save?idReponse=opaque-response",
        ] {
            assert_eq!(
                validate_questionnaire_url(&portal, invalid)
                    .unwrap_err()
                    .code,
                "invalid_questionnaire_request"
            );
        }
    }

    #[test]
    fn document_downloads_are_same_origin_and_read_only() {
        let portal = Url::parse("https://school.example/").unwrap();

        assert!(validate_document_url(
            &portal,
            "/Document/DownloadCertificatScolarite?IdInscriptionPeriode=opaque"
        )
        .is_ok());
        for invalid in [
            "https://evil.example/Document/DownloadCertificatScolarite?id=1",
            "//evil.example/Document/DownloadCertificatScolarite?id=1",
            "/Profil/ModificationAdresse",
            "/Document/DownloadFacture?IdFacture=1",
        ] {
            assert_eq!(
                validate_document_url(&portal, invalid).unwrap_err().code,
                "invalid_document_request"
            );
        }
    }

    #[test]
    fn document_chunks_stop_at_the_configured_limit() {
        let mut body = Vec::new();

        append_document_chunk(&mut body, b"1234", 5).unwrap();
        assert_eq!(
            append_document_chunk(&mut body, b"56", 5).unwrap_err().code,
            "document_too_large"
        );
        assert_eq!(body, b"1234");
    }

    #[test]
    fn parses_absence_periods_blocks_and_excused_column() {
        let portal = Url::parse("https://school.example/").unwrap();
        let html = r##"
            <main>
                <h1>Mes absences</h1>
                <div class="panel-group" id="accordion-periode">
                    <div class="panel panel-default">
                        <div class="panel-heading"><h4 class="panel-title">
                            <a data-toggle="collapse" href="#periode1">2025/2026</a>
                        </h4></div>
                        <div id="periode1" class="panel-collapse collapse"><div class="panel-body">
                            <div class="panel-group" id="accordion">
                                <div class="panel panel-default">
                                    <div class="panel-heading"><h4 class="panel-title">
                                        <a href="#4916038">2025/26 LI-B-ESCEN N1 - BLOC 1 (01/09/2025 - 31/07/2026)</a>
                                    </h4></div>
                                    <div id="4916038" class="panel-collapse collapse"><div class="panel-body">
                                        <p><a href="/Absence/DownloadReleveAbsence?IdSequence=4902899">Relevé d'absence</a></p>
                                        <table class="table table-striped">
                                            <thead><tr>
                                                <th>Date de la séance</th><th>Cours</th>
                                                <th>Durée</th><th>Excusée</th><th>Motif</th>
                                            </tr></thead>
                                            <tbody>
                                                <tr>
                                                    <td class="center">29/09/2025 &nbsp; 08:45</td>
                                                    <td>C00000101-1 Marketing : fondamentaux</td>
                                                    <td>3,25</td><td>Oui</td><td>Motif administratif</td>
                                                </tr>
                                                <tr>
                                                    <td class="center">30/09/2025</td>
                                                    <td>C00000102-1 Droit</td>
                                                    <td>1,5</td><td>Non</td><td></td>
                                                </tr>
                                                <tr>
                                                    <td class="center">01/10/2025 &nbsp; 14:00</td>
                                                    <td>C00000103-1 Anglais</td>
                                                    <td>2</td><td></td><td>Dossier transmis</td>
                                                </tr>
                                            </tbody>
                                        </table>
                                    </div></div>
                                </div>
                            </div>
                        </div></div>
                    </div>
                </div>
            </main>
        "##;

        let page = parse_portal_page(PortalResource::Absences, &portal, html, 1_777_777_777_777);

        assert!(page.markup_recognized);
        assert_eq!(page.absence_periods.len(), 1);
        let period = &page.absence_periods[0];
        assert_eq!(period.label, "2025/2026");
        assert_eq!(period.blocks.len(), 1);

        let block = &period.blocks[0];
        assert_eq!(
            block.label,
            "2025/26 LI-B-ESCEN N1 - BLOC 1 (01/09/2025 - 31/07/2026)"
        );
        assert_eq!(
            block.report_path.as_deref(),
            Some("/Absence/DownloadReleveAbsence?IdSequence=4902899")
        );
        assert_eq!(block.entries.len(), 3);

        let excused = &block.entries[0];
        assert_eq!(excused.date, "29/09/2025");
        assert_eq!(excused.time.as_deref(), Some("08:45"));
        assert_eq!(excused.course, "C00000101-1 Marketing : fondamentaux");
        assert_eq!(excused.duration.as_deref(), Some("3,25"));
        assert_eq!(excused.excused, Some(true));
        assert_eq!(excused.reason.as_deref(), Some("Motif administratif"));

        assert_eq!(block.entries[1].time, None);
        assert_eq!(block.entries[1].excused, Some(false));
        assert_eq!(block.entries[1].reason, None);

        // An empty column is a decision the school has not taken yet.
        assert_eq!(block.entries[2].excused, None);
    }

    #[test]
    fn flags_unrecognized_markup_without_guessing_empty_data() {
        let portal = Url::parse("https://school.example/").unwrap();
        let page = parse_portal_page(
            PortalResource::Absences,
            &portal,
            "<main><h1>Mes absences</h1><p>Aucune donnée</p></main>",
            1_777_777_777_777,
        );

        assert_eq!(page.title, "Mes absences");
        assert!(!page.markup_recognized);
    }

    #[test]
    fn parses_the_year_block_course_accordion_of_the_grades_page() {
        let portal = Url::parse("https://school.example/").unwrap();
        let html = r##"
            <div id="accordion-periode" class="panel-group">
              <div class="panel panel-default">
                <div class="panel-heading"><h4 class="panel-title"><a href="#periode1">2025/2026</a></h4></div>
                <div id="periode1" class="panel-collapse collapse in">
                  <div id="accordion" class="panel-group">
                    <div class="panel panel-default">
                      <div class="panel-heading">
                        <h4 class="panel-title"><a href="#4902901">2025/26 LI-B-ESCEN N1 - BLOC 1 (01/09/2025 - 31/07/2026)</a></h4>
                        <a href="/Note/DownloadBulletin?IdInscriptionSequence=4916038"> Bulletin de notes</a>
                        <a href="/Note/DownloadReleve?IdInscriptionSequence=4916038"> Relevé de notes</a>
                      </div>
                      <div id="4902901" class="panel-collapse collapse in">
                        <div class="tiles-list clearfix">
                          <div class="col-xs-12"><h4></h4><hr></div>
                          <div class="tile-container col-xs-12">
                            <div class="panel panel-default tile">
                              <div class="panel-heading">
                                <strong>C00000201-1 - Anglais</strong>
                                <a class="btn-link pop" href="/SaisieCahierTexte/IndexApprenant?idInscription=4916238"><i class="glyphicon"></i></a>
                              </div>
                              <div class="panel-body">
                                <dl class="dl-horizontal">
                                  <dt style="width:80px;"> <span>18,90/20</span></dt>
                                  <dd> Evaluation <small> (30,00%) </small> </dd>
                                  <dt style="width:80px;"> • <span>16,50/20</span></dt>
                                  <dd>Notes 10 octobre <small>(Pondération : 20,00)</small></dd>
                                  <dt style="width:80px;"> <span>15,40/20</span></dt>
                                  <dd> Partiel <small> (70,00%) </small> </dd>
                                </dl>
                              </div>
                            </div>
                          </div>
                          <div class="col-xs-12"><h4>LXP Learning Track (LXPLT)</h4><hr></div>
                          <div class="tile-container col-xs-12">
                            <div class="panel panel-default tile">
                              <div class="panel-heading"><strong>C00000202-1 - E-learning</strong></div>
                              <div class="panel-body"><dl class="dl-horizontal"><dt style="width:80px;"></dt><dd></dd></dl></div>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
        "##;

        let page = parse_portal_page(PortalResource::Grades, &portal, html, 1);

        assert_eq!(page.grade_periods.len(), 1);
        let period = &page.grade_periods[0];
        assert_eq!(period.label, "2025/2026");
        assert_eq!(period.blocks.len(), 1);

        let block = &period.blocks[0];
        assert_eq!(
            block.label,
            "2025/26 LI-B-ESCEN N1 - BLOC 1 (01/09/2025 - 31/07/2026)"
        );
        assert_eq!(
            block.bulletin_path.as_deref(),
            Some("/Note/DownloadBulletin?IdInscriptionSequence=4916038")
        );
        assert_eq!(
            block.transcript_path.as_deref(),
            Some("/Note/DownloadReleve?IdInscriptionSequence=4916038")
        );
        assert_eq!(block.sections.len(), 2);
        assert_eq!(block.sections[0].label, None);
        assert_eq!(
            block.sections[1].label.as_deref(),
            Some("LXP Learning Track (LXPLT)")
        );

        let course = &block.sections[0].courses[0];
        assert_eq!(course.code.as_deref(), Some("C00000201-1"));
        assert_eq!(course.name, "Anglais");
        assert_eq!(
            course.notebook_path.as_deref(),
            Some("/SaisieCahierTexte/IndexApprenant?idInscription=4916238")
        );
        assert_eq!(course.evaluations.len(), 2);
        assert_eq!(course.evaluations[0].label, "Evaluation");
        assert_eq!(course.evaluations[0].score.as_deref(), Some("18,90"));
        assert_eq!(course.evaluations[0].scale.as_deref(), Some("20"));
        assert_eq!(course.evaluations[0].weight.as_deref(), Some("30,00%"));
        assert_eq!(course.evaluations[0].children.len(), 1);
        assert_eq!(course.evaluations[0].children[0].label, "Notes 10 octobre");
        assert_eq!(
            course.evaluations[0].children[0].weight.as_deref(),
            Some("Pondération : 20,00")
        );
        assert_eq!(course.evaluations[1].label, "Partiel");

        // A course with no evaluation yet still belongs to its block.
        let ungraded = &block.sections[1].courses[0];
        assert_eq!(ungraded.name, "E-learning");
        assert!(ungraded.evaluations.is_empty());

        let grades = extract_grades(&page);
        assert_eq!(grades.len(), 3);
        assert_eq!(grades[0].subject, "Anglais");
        assert_eq!(grades[1].label, "Notes 10 octobre");
        assert_eq!(grades[2].score, "15,40");
    }

    #[test]
    fn extracts_typed_grades_from_recognized_columns() {
        let portal = Url::parse("https://school.example/").unwrap();
        let page = parse_portal_page(
            PortalResource::Grades,
            &portal,
            r#"<h2>Mathématiques</h2><table><tr><th>Libellé</th><th>Note</th><th>Coeff.</th><th>Moyenne</th></tr><tr><td>Partiel</td><td>16 / 20</td><td>2</td><td>13,5</td></tr></table>"#,
            1,
        );

        let grades = extract_grades(&page);

        assert_eq!(grades.len(), 1);
        assert_eq!(grades[0].subject, "Mathématiques");
        assert_eq!(grades[0].label, "Partiel");
        assert_eq!(grades[0].score, "16");
        assert_eq!(grades[0].scale.as_deref(), Some("20"));
        assert_eq!(grades[0].coefficient.as_deref(), Some("2"));
        assert_eq!(grades[0].average.as_deref(), Some("13,5"));
    }

    #[test]
    fn extracts_home_grades_from_the_latest_school_year_only() {
        let portal = Url::parse("https://school.example/").unwrap();
        let html = r##"
            <div id="accordion-periode" class="panel-group">
              <div class="panel panel-default">
                <div class="panel-heading"><h4 class="panel-title"><a href="#old">2025/2026</a></h4></div>
                <div id="old" class="panel-collapse"><div class="panel-group"><div class="panel panel-default">
                  <div class="panel-heading"><h4 class="panel-title">Old year</h4></div>
                  <div class="tiles-list"><div class="tile-container"><div class="tile"><div class="panel-heading"><strong>OLD - History</strong></div>
                    <dl><dt>10/20</dt><dd>Exam</dd></dl>
                  </div></div></div>
                </div></div></div>
              </div>
              <div class="panel panel-default">
                <div class="panel-heading"><h4 class="panel-title"><a href="#new">2026/2027</a></h4></div>
                <div id="new" class="panel-collapse"><div class="panel-group"><div class="panel panel-default">
                  <div class="panel-heading"><h4 class="panel-title">New year</h4></div>
                  <div class="tiles-list"><div class="tile-container"><div class="tile"><div class="panel-heading"><strong>NEW - Mathematics</strong></div>
                    <dl><dt>18/20</dt><dd>Assignment</dd></dl>
                  </div></div></div>
                </div></div></div>
              </div>
            </div>
        "##;

        let page = parse_portal_page(PortalResource::Grades, &portal, html, 1);
        let grades = extract_latest_grades(&page);

        assert_eq!(grades.len(), 1);
        assert_eq!(grades[0].subject, "Mathematics");
        assert_eq!(grades[0].score, "18");
    }
}
