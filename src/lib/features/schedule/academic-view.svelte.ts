import { downloadPortalDocument, periodStartYear } from './portal-utils';
import type { PortalDocument } from './types';

/**
 * The state the two year/block academic views — grades and absences — keep
 * beside their portal resource. Identical in both, so it lives here once.
 */

/**
 * The current school year is the latest one the portal publishes — it does not
 * list a year before it starts, and it does not always list the newest first.
 */
export function latestPeriod<T extends { label: string }>(periods: T[]): T | null {
  return periods.reduce<T | null>(
    (latest, period) =>
      latest === null || periodStartYear(period.label) > periodStartYear(latest.label)
        ? period
        : latest,
    null
  );
}

export type BlockDisclosure = {
  /** The blocks unfolded in the given school year. */
  openIds: (periodId: string | null | undefined) => string[];
  toggle: (periodId: string | null | undefined, blockId: string) => void;
};

/**
 * Which blocks are unfolded, tracked per school year so switching years does
 * not reshuffle either. Every block starts collapsed: the list of blocks is the
 * overview, and one block unfolded on arrival buries the others below the fold.
 */
export function createBlockDisclosure(): BlockDisclosure {
  let openByPeriod = $state<Record<string, string[]>>({});

  return {
    openIds(periodId) {
      return periodId ? (openByPeriod[periodId] ?? []) : [];
    },
    toggle(periodId, blockId) {
      if (!periodId) return;
      const open = openByPeriod[periodId] ?? [];
      openByPeriod = {
        ...openByPeriod,
        [periodId]: open.includes(blockId)
          ? open.filter((id) => id !== blockId)
          : [...open, blockId],
      };
    },
  };
}

export type DocumentDownload = {
  /** The request path currently being fetched, so only its own pill goes busy. */
  readonly requestPath: string | null;
  /** The last download failed; the view states it rather than swallowing it. */
  readonly failed: boolean;
  /**
   * Where the last document landed. The file is saved even when the system
   * declines to open it, so the view has to name the path either way.
   */
  readonly savedPath: string | null;
  download: (document: PortalDocument) => Promise<void>;
};

export function createDocumentDownload(): DocumentDownload {
  let requestPath = $state<string | null>(null);
  let failed = $state(false);
  let savedPath = $state<string | null>(null);

  async function download(document: PortalDocument) {
    requestPath = document.requestPath;
    failed = false;
    savedPath = null;
    try {
      const result = await downloadPortalDocument(document);
      savedPath = result.path;
    } catch {
      failed = true;
    } finally {
      requestPath = null;
    }
  }

  return {
    get requestPath() {
      return requestPath;
    },
    get failed() {
      return failed;
    },
    get savedPath() {
      return savedPath;
    },
    download,
  };
}
