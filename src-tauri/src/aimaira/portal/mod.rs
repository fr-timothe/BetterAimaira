//! Reading the pages of the Aimaira student portal.
//!
//! One module per page the app knows how to read, plus the pieces they share:
//! `model` holds the shapes handed to the interface, `html` the readers every
//! parser calls, `selectors` the compiled CSS, and `tables` the generic
//! `<table>` and label/value readers the specialised pages fall back on.

use reqwest::{Client, Url};
use scraper::Html;
use serde::{Deserialize, Serialize};

use crate::error::CommandError;

use super::{current_timestamp_millis, is_login_document, is_login_page};

mod absences;
mod documents;
mod grades;
mod html;
mod model;
mod questionnaires;
mod selectors;
mod tables;

pub use documents::download_portal_document;
pub use grades::{extract_grades, extract_latest_grades};
// Only the three types the rest of the crate names. The others are reached
// through the fields of `PortalPage`, so re-exporting them here would only add
// names nothing outside `portal` ever writes.
pub use model::{Grade, PortalPage, QuestionnaireDetail};

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
    /// The name a stored snapshot is filed under. Deliberately not `route()`:
    /// that is a URL path the portal owns and may rename, which would silently
    /// orphan every row already on disk.
    pub fn key(self) -> &'static str {
        match self {
            Self::Grades => "grades",
            Self::Absences => "absences",
            Self::Profile => "profile",
            Self::Documents => "documents",
            Self::Questionnaires => "questionnaires",
        }
    }

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
    let endpoint = questionnaires::validate_questionnaire_url(portal_url, response_path)?;
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

    questionnaires::parse_questionnaire_detail(&document)
        .ok_or_else(|| CommandError::new("questionnaire_invalid_response"))
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
    let title = html::select_first_text(
        document,
        &[
            &selectors::MAIN_HEADING,
            &selectors::MAIN_SUBHEADING,
            &selectors::DOCUMENT_HEADING,
            &selectors::DOCUMENT_TITLE,
        ],
    )
    .unwrap_or_default();
    let (headings, tables) = tables::parse_semantic_tables(document);
    // Both readers are generic enough to answer on any page, and answering everywhere is the
    // problem: the grades accordion is built from `<dl>` pairs, so every evaluation used to
    // leave as a profile field, and the bulletin links the grade blocks already carry used to
    // be counted again here. Both then propped `markup_recognized` up, hiding a grades page
    // whose real parsing had stopped working.
    let fields = match resource {
        PortalResource::Profile => tables::parse_fields(document),
        _ => Vec::new(),
    };
    let documents = match resource {
        PortalResource::Documents => documents::parse_documents(document, portal_url),
        _ => Vec::new(),
    };
    let grade_periods = match resource {
        PortalResource::Grades => grades::parse_grade_periods(document, portal_url),
        _ => Vec::new(),
    };
    let absence_periods = match resource {
        PortalResource::Absences => absences::parse_absence_periods(document, portal_url),
        _ => Vec::new(),
    };
    let questionnaires = match resource {
        PortalResource::Questionnaires => {
            questionnaires::parse_questionnaires(document, portal_url)
        }
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
        stale: false,
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

#[cfg(test)]
mod tests {
    use super::model::PortalDocumentKind;
    use super::{parse_portal_page, PortalResource};
    use reqwest::Url;

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
        assert!(page.markup_recognized);

        // Fields and documents belong to the pages that publish them, not to every page
        // whose markup happens to contain a `<dl>` or a download link.
        let profile = parse_portal_page(PortalResource::Profile, &portal, html, 1);
        assert_eq!(profile.fields.len(), 2);

        let documents = parse_portal_page(PortalResource::Documents, &portal, html, 1);
        assert_eq!(documents.documents.len(), 1);
        assert_eq!(
            documents.documents[0].kind,
            PortalDocumentKind::GradeBulletin
        );
        assert_eq!(
            documents.documents[0].request_path,
            "/Note/DownloadBulletin?IdInscriptionSequence=opaque"
        );
    }

    #[test]
    fn keeps_grade_markup_out_of_the_profile_fields_and_the_document_list() {
        let portal = Url::parse("https://school.example/").unwrap();
        // A grades page whose accordion no longer parses: the `<dl>` evaluations and the
        // bulletin link used to answer in its place and keep `markup_recognized` green.
        let html = r#"
            <main>
                <h1>Mes notes</h1>
                <div class="tile">
                    <dl><dt>18,90/20</dt><dd>Evaluation</dd></dl>
                    <a href="/Note/DownloadBulletin?IdInscriptionSequence=opaque">Bulletin</a>
                </div>
            </main>
        "#;

        let page = parse_portal_page(PortalResource::Grades, &portal, html, 1);

        assert!(page.fields.is_empty());
        assert!(page.documents.is_empty());
        assert!(page.grade_periods.is_empty());
        assert!(!page.markup_recognized);
    }

    #[test]
    fn reads_the_page_heading_rather_than_the_browser_tab_title() {
        let portal = Url::parse("https://school.example/").unwrap();
        let html = r#"
            <html>
                <head><title>Aimaira</title></head>
                <body><main><h1>Liste des Questionnaires</h1></main></body>
            </html>
        "#;

        let page = parse_portal_page(PortalResource::Questionnaires, &portal, html, 1);

        assert_eq!(page.title, "Liste des Questionnaires");
        // An empty questionnaire list is an honest answer, not a portal that changed markup.
        assert!(page.questionnaires.is_empty());
        assert!(page.markup_recognized);
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
}
