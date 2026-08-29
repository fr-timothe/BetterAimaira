//! The grades page: the year → block → course accordion, and the flat list of
//! graded work the home screen and the alert history are built from.

use std::collections::HashSet;

use reqwest::Url;
use scraper::{ElementRef, Html};

use super::documents::document_kind;
use super::html::{element_text, request_path, same_origin_url};
use super::model::{
    Grade, GradeBlock, GradeCourse, GradeEvaluation, GradePeriod, GradeSection, PortalDocumentKind,
    PortalPage,
};
use super::selectors;
use super::tables::find_header;
use crate::aimaira::stable_hash_hex;

/// The grades page ships no table: years, blocks and courses are Bootstrap
/// accordion panels, and each course lists its evaluations in a `<dl>`.
pub(super) fn parse_grade_periods(document: &Html, portal_url: &Url) -> Vec<GradePeriod> {
    let mut periods = Vec::new();

    for period in document.select(&selectors::PERIOD_PANEL) {
        let label = period
            .select(&selectors::PANEL_HEADING)
            .next()
            .and_then(|heading| heading.select(&selectors::PANEL_TITLE).next())
            .map(element_text)
            .unwrap_or_default();
        let mut blocks = Vec::new();

        for block in period.select(&selectors::BLOCK_PANEL) {
            let heading = block.select(&selectors::PANEL_HEADING).next();
            let block_label = heading
                .and_then(|heading| heading.select(&selectors::PANEL_TITLE).next())
                .map(element_text)
                .unwrap_or_default();
            let mut bulletin_path = None;
            let mut transcript_path = None;
            if let Some(heading) = heading {
                for anchor in heading.select(&selectors::ANCHOR) {
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
            if let Some(tiles) = block.select(&selectors::TILES_LIST).next() {
                for child in tiles.children().filter_map(ElementRef::wrap) {
                    if let Some(tile) = child.select(&selectors::TILE).next() {
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
                        .select(&selectors::HEADING)
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
    let title = tile
        .select(&selectors::TILE_TITLE)
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
        .select(&selectors::TILE_ANCHOR)
        .filter_map(|anchor| same_origin_url(portal_url, anchor.value().attr("href")))
        .find(|url| {
            url.path()
                .to_ascii_lowercase()
                .starts_with("/saisiecahiertexte")
        })
        .map(|url| request_path(&url));

    let mut evaluations = Vec::<GradeEvaluation>::new();
    let mut pending_score: Option<(bool, Option<String>, Option<String>)> = None;
    for item in tile.select(&selectors::DEFINITION_ITEM) {
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
                let weight = item.select(&selectors::SMALL).next().map(element_text);
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
/// This is also what gets stored for offline replay, so a reader who loses the
/// portal sees the same school year they were shown online. Falls back to the
/// full list when no period carries a year the app can read.
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

fn collect_period_grades(
    period: &GradePeriod,
    grades: &mut Vec<Grade>,
    seen: &mut HashSet<String>,
) {
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

fn split_score(value: &str) -> (String, Option<String>) {
    let Some((score, scale)) = value.split_once('/') else {
        return (value.trim().to_owned(), None);
    };
    let score = score.trim().to_owned();
    let scale = scale.trim().to_owned();
    (score, (!scale.is_empty()).then_some(scale))
}

#[cfg(test)]
mod tests {
    use super::{extract_grades, extract_latest_grades};
    use crate::aimaira::portal::{parse_portal_page, PortalResource};
    use reqwest::Url;

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
