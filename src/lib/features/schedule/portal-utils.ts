import { invoke } from '@tauri-apps/api/core';
import * as m from '$lib/paraglide/messages.js';
import type {
  PortalDocument,
  PortalDocumentKind,
  PortalResourceErrorCode,
} from './types';

export const PASTEL_PALETTE = [
  '#DDEFFF',
  '#E8F8D7',
  '#FCE5F2',
  '#FFF3D6',
  '#EADCF8',
];

/** The portal names a school year the same way on the grades and absences pages. */
const DATE_RANGE = /\(\s*(\d{2}\/\d{2}\/\d{4})\s*-\s*(\d{2}\/\d{2}\/\d{4})\s*\)\s*$/;
const LEADING_YEAR = /^\d{4}\/\d{2,4}\s+/;

/** `2025/2026` → 2025. Used to land on the current school year, not the first listed. */
export function periodStartYear(label: string): number {
  const match = label.match(/(\d{4})/);
  return match ? Number.parseInt(match[1], 10) : Number.NEGATIVE_INFINITY;
}

/**
 * `2025/26 LI-B-ESCEN N1 - BLOC 1 (01/09/2025 - 31/07/2026)` carries three
 * things at once. Split them instead of truncating: the dates go to a subtitle
 * and the school year is already the enclosing period.
 */
export function splitBlockLabel(label: string): { title: string; range: string | null } {
  const match = label.match(DATE_RANGE);
  const range = match ? `${match[1]} – ${match[2]}` : null;
  const title = label.replace(DATE_RANGE, '').replace(LEADING_YEAR, '').trim();
  return { title: title || label, range };
}

export function normalizeText(text: string): string {
  return text
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .trim();
}

export function parseNumber(val: string): number | null {
  if (!val) return null;
  const clean = val.replace(',', '.').replace(/[^0-9.-]/g, '').trim();
  const num = parseFloat(clean);
  return Number.isNaN(num) ? null : num;
}

export function parseDurationHours(str: string): number {
  if (!str) return 1;
  const clean = str.toLowerCase().trim();
  const matchHMin = clean.match(/^([0-9]+)\s*h\s*([0-9]+)?/);
  if (matchHMin) {
    const hours = parseInt(matchHMin[1], 10);
    const mins = matchHMin[2] ? parseInt(matchHMin[2], 10) : 0;
    return hours + mins / 60;
  }
  const matchColon = clean.match(/^([0-9]{1,2}):([0-9]{2})/);
  if (matchColon) {
    const hours = parseInt(matchColon[1], 10);
    const mins = parseInt(matchColon[2], 10);
    return hours + mins / 60;
  }
  const matchNum = clean.match(/^([0-9]+(?:[.,][0-9]+)?)/);
  if (matchNum) {
    return parseFloat(matchNum[1].replace(',', '.'));
  }
  return 1;
}

export function getSubjectColor(subject: string): string {
  let hash = 0;
  for (let i = 0; i < subject.length; i++) {
    hash = (hash * 31 + subject.charCodeAt(i)) & 0xffffffff;
  }
  const index = Math.abs(hash) % PASTEL_PALETTE.length;
  return PASTEL_PALETTE[index];
}

export function documentKindLabel(kind: PortalDocumentKind): string {
  switch (kind) {
    case 'absenceReport':
      return m.document_kind_absence_report();
    case 'gradeBulletin':
      return m.document_kind_grade_bulletin();
    case 'gradeTranscript':
      return m.document_kind_grade_transcript();
    case 'enrollmentCertificate':
      return m.document_kind_enrollment_certificate();
    case 'schoolCertificate':
      return m.document_kind_school_certificate();
    case 'schoolTranscript':
      return m.document_kind_school_transcript();
    case 'gradeReport':
      return m.document_kind_grade_report();
    default:
      return 'Document';
  }
}

export function documentFilename(document: PortalDocument, fallback = 'document'): string {
  const requestedName =
    document.suggestedFilename?.trim() || document.label.trim() || fallback;
  const safeName = requestedName.replace(/[<>:"/\\|?*\u0000-\u001f]/g, '-');
  return safeName.toLowerCase().endsWith('.pdf') ? safeName : `${safeName}.pdf`;
}

export async function downloadPortalDocument(document: PortalDocument): Promise<void> {
  const bytes = await invoke<ArrayBuffer>('download_portal_document', {
    request: { requestPath: document.requestPath },
  });
  const blob = new Blob([bytes], { type: 'application/pdf' });
  const objectUrl = URL.createObjectURL(blob);
  const anchor = window.document.createElement('a');
  anchor.href = objectUrl;
  anchor.download = documentFilename(document);
  anchor.click();
  URL.revokeObjectURL(objectUrl);
}

export function parseResourceError(
  error: unknown,
  fallback: PortalResourceErrorCode = 'internal_error'
): PortalResourceErrorCode {
  if (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    typeof error.code === 'string'
  ) {
    switch (error.code) {
      case 'session_expired':
      case 'grades_unavailable':
      case 'absences_unavailable':
      case 'profile_unavailable':
      case 'documents_unavailable':
      case 'questionnaires_unavailable':
      case 'questionnaire_invalid_response':
      case 'invalid_questionnaire_request':
      case 'internal_error':
        return error.code;
    }
  }
  return fallback;
}

export function resourceErrorMessage(code: PortalResourceErrorCode): string {
  switch (code) {
    case 'session_expired':
      return m.resource_session_expired();
    case 'grades_unavailable':
      return m.resource_grades_unavailable();
    case 'absences_unavailable':
      return m.resource_absences_unavailable();
    case 'profile_unavailable':
      return m.resource_profile_unavailable();
    case 'documents_unavailable':
      return m.resource_documents_unavailable();
    case 'questionnaires_unavailable':
      return m.resource_questionnaires_unavailable();
    case 'questionnaire_invalid_response':
      return m.resource_questionnaire_invalid_response();
    case 'invalid_questionnaire_request':
      return m.resource_questionnaire_invalid_request();
    case 'internal_error':
    default:
      return m.resource_generic_error();
  }
}

/**
 * `locale` is not read here — the fallback comes from the message catalogue,
 * which resolves the active locale itself. It stays in the signature so the
 * callers' `$derived` keeps a dependency on the locale and re-runs on a switch.
 */
export function getDisplayName(username: string, locale = 'fr'): string {
  if (!username) return m.student_display_fallback();
  if (username.includes('@')) {
    const localPart = username.split('@')[0];
    const parts = localPart.split(/[._-]/);
    return parts.map((p) => p.charAt(0).toUpperCase() + p.slice(1).toLowerCase()).join(' ');
  }
  return username;
}

export function getPortalHost(portalUrl: string): string {
  try {
    const url = new URL(portalUrl.startsWith('http') ? portalUrl : `https://${portalUrl}`);
    return url.hostname;
  } catch {
    return portalUrl;
  }
}
