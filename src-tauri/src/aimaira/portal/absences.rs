//! The absences page: the same year → block accordion as the grades page, with
//! the missed sessions of each block held in ordinary tables.

use reqwest::Url;
use scraper::Html;

use super::documents::document_kind;
use super::html::{element_text, request_path, same_origin_url};
use super::model::{AbsenceBlock, AbsenceEntry, AbsencePeriod, PortalDocumentKind};
use super::selectors;
use super::tables::{find_header, normalize_header};
use crate::aimaira::stable_hash_hex;

/// The absences page reuses the grades layout: year → block Bootstrap
/// accordions, each block carrying its own attendance report link and its
/// tables of missed sessions. Nothing in the markup promises there is only one:
/// reading just the first would drop the rest without failing a single guard, so
/// every table of the block is read.
pub(super) fn parse_absence_periods(document: &Html, portal_url: &Url) -> Vec<AbsencePeriod> {
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
            let block_label = block
                .select(&selectors::PANEL_HEADING)
                .next()
                .and_then(|heading| heading.select(&selectors::PANEL_TITLE).next())
                .map(element_text)
                .unwrap_or_default();
            // The report link sits in the block body, above the table.
            let report_path = block
                .select(&selectors::ANCHOR)
                .filter_map(|anchor| same_origin_url(portal_url, anchor.value().attr("href")))
                .find(|url| document_kind(url.path()) == Some(PortalDocumentKind::AbsenceReport))
                .map(|url| request_path(&url));

            let block_id = stable_hash_hex(&[&label, &block_label]);
            let mut entries = Vec::new();
            // Row numbering runs across the tables of a block so entry ids stay unique, and
            // stay what they were for the single-table blocks that are the common case.
            let mut next_row_index = 0_usize;

            for table in block.select(&selectors::TABLE) {
                let headers = table
                    .select(&selectors::TABLE_HEAD_CELL)
                    .map(element_text)
                    .collect::<Vec<_>>();
                let date_column = find_header(&headers, &["date", "seance", "jour"]);
                let course_column = find_header(&headers, &["cours", "matiere", "libelle"]);
                let duration_column = find_header(&headers, &["duree", "heure", "volume"]);
                let excused_column =
                    find_header(&headers, &["excuse", "justifi", "statut", "etat"]);
                let reason_column = find_header(&headers, &["motif", "raison", "commentaire"]);

                for row in table.select(&selectors::TABLE_BODY_ROW) {
                    let row_index = next_row_index;
                    next_row_index += 1;
                    let cells = row
                        .select(&selectors::TABLE_DATA_CELL)
                        .map(element_text)
                        .collect::<Vec<_>>();
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

#[cfg(test)]
mod tests {
    use crate::aimaira::portal::{parse_portal_page, PortalResource};
    use reqwest::Url;

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
    fn reads_every_table_of_an_absence_block() {
        let portal = Url::parse("https://school.example/").unwrap();
        let html = r##"
            <main>
                <div class="panel-group" id="accordion-periode">
                    <div class="panel panel-default">
                        <div class="panel-heading"><h4 class="panel-title">2025/2026</h4></div>
                        <div class="panel-group">
                            <div class="panel panel-default">
                                <div class="panel-heading"><h4 class="panel-title">BLOC 1</h4></div>
                                <table>
                                    <thead><tr><th>Date de la séance</th><th>Cours</th></tr></thead>
                                    <tbody><tr><td>29/09/2025 08:45</td><td>Marketing</td></tr></tbody>
                                </table>
                                <table>
                                    <thead><tr><th>Date de la séance</th><th>Cours</th></tr></thead>
                                    <tbody><tr><td>30/09/2025 10:00</td><td>Droit</td></tr></tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            </main>
        "##;

        let page = parse_portal_page(PortalResource::Absences, &portal, html, 1);

        let block = &page.absence_periods[0].blocks[0];
        assert_eq!(block.entries.len(), 2);
        assert_eq!(block.entries[0].course, "Marketing");
        assert_eq!(block.entries[1].course, "Droit");
        assert_ne!(block.entries[0].id, block.entries[1].id);
    }
}
