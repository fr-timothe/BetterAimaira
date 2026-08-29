//! The downloadable PDFs: which portal routes count as one, which links a page
//! publishes, and the streaming download itself.

use std::collections::HashSet;

use futures_util::StreamExt;
use reqwest::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Client, Url,
};
use scraper::Html;

use super::html::{element_text, request_path};
use super::model::{PortalDocument, PortalDocumentKind};
use super::selectors;
use crate::aimaira::is_login_page;
use crate::error::CommandError;

const MAX_DOCUMENT_BYTES: u64 = 25 * 1024 * 1024;

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

pub(super) fn parse_documents(document: &Html, portal_url: &Url) -> Vec<PortalDocument> {
    let mut seen = HashSet::new();
    let mut documents = Vec::new();

    for anchor in document.select(&selectors::ANCHOR) {
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

pub(super) fn document_kind(path: &str) -> Option<PortalDocumentKind> {
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

#[cfg(test)]
mod tests {
    use super::{append_document_chunk, validate_document_url};
    use reqwest::Url;

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
}
