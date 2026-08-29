//! The questionnaires page and one response of it. The list is ordinary
//! markup; the response itself is a SurveyJS definition the portal inlines as
//! JSON, with the saved answers in a second variable beside it.

use std::collections::HashSet;

use reqwest::Url;
use scraper::{ElementRef, Html};
use serde_json::Value;

use super::html::{
    element_text, extract_script_json, json_scalar_text, request_path, same_origin_url,
};
use super::model::{
    QuestionnaireDetail, QuestionnaireOption, QuestionnairePage, QuestionnaireQuestion,
    QuestionnaireSummary,
};
use super::selectors;
use super::tables::normalize_header;
use crate::aimaira::{html_to_text, stable_hash_hex};
use crate::error::CommandError;

pub(super) fn parse_questionnaires(document: &Html, portal_url: &Url) -> Vec<QuestionnaireSummary> {
    let mut questionnaires = Vec::new();
    let mut seen = HashSet::new();

    for anchor in document.select(&selectors::QUESTIONNAIRE_RESPONSE_ANCHOR) {
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
            .select(&selectors::QUESTIONNAIRE_TITLE)
            .next()
            .map(element_text)
            .unwrap_or_default();
        let context = row
            .select(&selectors::QUESTIONNAIRE_CONTEXT)
            .next()
            .map(element_text)
            .unwrap_or_default()
            .strip_prefix(&title)
            .unwrap_or_default()
            .trim()
            .to_owned();
        let deadline = row
            .select(&selectors::QUESTIONNAIRE_DEADLINE)
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
            .select(&selectors::LABEL)
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

pub(super) fn parse_questionnaire_detail(document: &Html) -> Option<QuestionnaireDetail> {
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
    let page_name = page.get("name").and_then(Value::as_str).unwrap_or_default();
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
                    format!(
                        "{}: {}",
                        html_to_text(key),
                        questionnaire_answer_label(&value, options)
                    )
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

fn questionnaire_is_complete(document: &Html) -> bool {
    document.select(&selectors::SCRIPT).any(|script| {
        let source = script.text().collect::<String>();
        source.contains("isComplete: true") || source.contains("isComplete:true")
    })
}

pub(super) fn validate_questionnaire_url(
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
    url.path()
        .trim_end_matches('/')
        .eq_ignore_ascii_case("/Questionnaire/Reponse")
        && url
            .query_pairs()
            .any(|(key, value)| key.eq_ignore_ascii_case("idReponse") && !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{parse_questionnaire_detail, validate_questionnaire_url};
    use crate::aimaira::portal::{parse_portal_page, PortalResource};
    use reqwest::Url;
    use scraper::Html;

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
}
