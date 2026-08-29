//! The shapes the portal parsers hand to the interface. Every one of them is
//! serialised, either over the Tauri bridge or into the on-disk snapshot, so a
//! renamed field is a breaking change for the frontend and for the stored rows.

use serde::{Deserialize, Serialize};

use super::PortalResource;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalPage {
    pub resource: PortalResource,
    /// When the portal answered, in milliseconds. A page replayed from disk
    /// keeps the timestamp of its original fetch so the interface can say how
    /// old the content is.
    pub fetched_at: u64,
    /// Raised when the page comes from the on-disk snapshot because the portal
    /// could not be reached.
    pub stale: bool,
    pub title: String,
    pub headings: Vec<String>,
    pub tables: Vec<PortalTable>,
    pub fields: Vec<PortalField>,
    pub documents: Vec<PortalDocument>,
    /// Only filled for the grades page: the portal nests grades in year →
    /// block → course accordions that carry no `<table>` at all.
    pub grade_periods: Vec<GradePeriod>,
    /// Only filled for the absences page: same year → block accordions, each
    /// block holding one or more tables of missed sessions.
    pub absence_periods: Vec<AbsencePeriod>,
    /// Only filled for the questionnaires page: one entry per portal response.
    pub questionnaires: Vec<QuestionnaireSummary>,
    pub markup_recognized: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradePeriod {
    pub id: String,
    pub label: String,
    pub blocks: Vec<GradeBlock>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeBlock {
    pub id: String,
    pub label: String,
    pub bulletin_path: Option<String>,
    pub transcript_path: Option<String>,
    pub sections: Vec<GradeSection>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeSection {
    pub label: Option<String>,
    pub courses: Vec<GradeCourse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeCourse {
    pub id: String,
    pub code: Option<String>,
    pub name: String,
    pub notebook_path: Option<String>,
    pub evaluations: Vec<GradeEvaluation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsencePeriod {
    pub id: String,
    pub label: String,
    pub blocks: Vec<AbsenceBlock>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceBlock {
    pub id: String,
    pub label: String,
    pub report_path: Option<String>,
    pub entries: Vec<AbsenceEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalTable {
    pub context: Vec<String>,
    pub caption: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalField {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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
