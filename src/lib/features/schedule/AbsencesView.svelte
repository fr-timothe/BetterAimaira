<script lang="ts">
  import { onMount } from 'svelte';
  import {
    AlertCircle,
    CalendarX2,
    CheckCircle2,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    Clock,
    CloudOff,
    FileText,
    PartyPopper,
    RefreshCw,
    ShieldCheck,
    XCircle,
  } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import PageShell from '$lib/components/ui/PageShell.svelte';
  import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import { connectivity } from '$lib/state/connectivity.svelte';
  import HeroMetric from './HeroMetric.svelte';
  import HeroStat from './HeroStat.svelte';
  import AbsencesViewSkeleton from './AbsencesViewSkeleton.svelte';
  import {
    absenceStatus,
    absenceTotals,
    blockHours,
    calculatePresenceRate,
    periodEntries,
    sortEntriesByDateDesc,
  } from './absence-utils';
  import { loadPortalResource } from './portal-cache';
  import {
    downloadPortalDocument,
    parseResourceError,
    periodStartYear,
    resourceErrorMessage,
    splitBlockLabel,
  } from './portal-utils';
  import type { AbsenceBlock, PortalDocument, PortalResourceState } from './types';

  type Props = {
    locale: Locale;
    onLogout: () => Promise<void>;
    refresh?: () => Promise<void>;
  };

  let { locale, onLogout, refresh = $bindable() }: Props = $props();

  $effect(() => {
    refresh = () => loadAbsences(true);
  });

  let absencesState = $state<PortalResourceState>({ kind: 'loading' });
  let viewMode = $state<'cards' | 'table'>('cards');
  let selectedPeriodId = $state<string | null>(null);
  /** Open blocks per school year, so switching years does not reshuffle either. */
  let openBlocksByPeriod = $state<Record<string, string[]>>({});
  let downloadingPath = $state<string | null>(null);
  let downloadFailed = $state(false);
  let requestSequence = 0;
  let refreshing = $state(false);
  /** A refresh that failed while data was already on screen. */
  let refreshFailed = $state(false);

  const copy = $derived.by(() => {
    locale;
    return {
      heading: m.absences_heading(),
      totalHours: m.absence_total_hours(),
      presenceRate: m.absence_presence_rate(),
      sessionCount: m.absence_session_count(),
      excused: m.absence_excused(),
      unexcused: m.absence_unexcused(),
      pending: m.absence_pending(),
      loading: m.resource_loading(),
      refresh: m.resource_refresh(),
      errorHeading: m.resource_error_heading(),
      retry: m.resource_retry(),
      backToLogin: m.back_to_login(),
      emptyHeading: m.resource_empty_heading(),
      emptyDescription: m.resource_empty_description(),
      offlineHeading: m.sync_offline(),
      offlineDescription: m.sync_offline_description(),
      cardsView: m.cards_view(),
      tableView: m.table_view(),
      viewModeLabel: m.absence_view_mode_label(),
      yearLabel: m.absence_year_label(),
      previousYear: m.absence_previous_year(),
      nextYear: m.absence_next_year(),
      blockHours: m.absence_block_hours(),
      blockEmpty: m.absence_block_empty(),
      entryExcused: m.absence_entry_excused(),
      entryUnexcused: m.absence_entry_unexcused(),
      entryPending: m.absence_entry_pending(),
      reasonLabel: m.absence_reason_label(),
      report: m.document_kind_absence_report(),
      downloadFailed: m.absence_download_failed(),
      emptyTitle: m.absence_empty_title(),
      emptyYearDescription: m.absence_empty_description(),
      tableBlock: m.absence_table_block(),
      tableDate: m.absence_table_date(),
      tableCourse: m.absence_table_course(),
      tableDuration: m.absence_table_duration(),
      tableStatus: m.absence_table_status(),
      tableReason: m.absence_table_reason(),
    };
  });

  onMount(() => {
    void loadAbsences();
  });

  async function loadAbsences(force = false) {
    const sequence = ++requestSequence;
    const hasData = absencesState.kind === 'ready';
    if (hasData) refreshing = true;
    else absencesState = { kind: 'loading' };
    try {
      const page = await loadPortalResource('absences', force);
      if (sequence !== requestSequence) return;
      absencesState = { kind: 'ready', page };
      refreshFailed = false;
    } catch (error) {
      if (sequence !== requestSequence) return;
      // Keeping stale data on screen is fine; keeping it unmarked is not. Flag
      // the failure so the freshness label can say what the user is looking at.
      if (hasData) refreshFailed = true;
      else absencesState = { kind: 'error', code: parseResourceError(error, 'absences_unavailable') };
    } finally {
      if (sequence === requestSequence) refreshing = false;
    }
  }

  const fetchedAt = $derived(absencesState.kind === 'ready' ? absencesState.page.fetchedAt : null);

  const periods = $derived(absencesState.kind === 'ready' ? absencesState.page.absencePeriods : []);

  /**
   * The current school year is the latest one the portal publishes — it does
   * not list a year before it starts, and it does not always list the newest
   * first.
   */
  const latestPeriod = $derived(
    periods.reduce<(typeof periods)[number] | null>(
      (latest, period) =>
        latest === null || periodStartYear(period.label) > periodStartYear(latest.label)
          ? period
          : latest,
      null
    )
  );

  const selectedPeriod = $derived(
    periods.find((period) => period.id === selectedPeriodId) ?? latestPeriod
  );

  const blocks = $derived(selectedPeriod?.blocks ?? []);

  const periodEntryList = $derived(selectedPeriod ? periodEntries(selectedPeriod) : []);

  const totals = $derived(absenceTotals(periodEntryList));
  const presencePercentage = $derived(calculatePresenceRate(totals, blocks.length));

  // Every block starts collapsed: the list of blocks is the overview, and one
  // block unfolded on arrival buries the others below the fold.
  const openBlockIds = $derived(
    selectedPeriod ? (openBlocksByPeriod[selectedPeriod.id] ?? []) : []
  );

  function toggleBlock(blockId: string) {
    const period = selectedPeriod;
    if (!period) return;
    const open = openBlocksByPeriod[period.id] ?? [];
    openBlocksByPeriod = {
      ...openBlocksByPeriod,
      [period.id]: open.includes(blockId)
        ? open.filter((id) => id !== blockId)
        : [...open, blockId],
    };
  }

  function formatHours(hours: number): string {
    return hours.toLocaleString(locale, { maximumFractionDigits: 2 });
  }

  /** The portal's own wording for the cell, so a `3,25` stays `3,25 h`. */
  function formatDuration(duration: string | null): string {
    if (!duration) return '--';
    return /\d\s*(h|:)/i.test(duration) ? duration : `${duration} h`;
  }

  const viewOptions = $derived([
    { value: 'cards', label: copy.cardsView },
    { value: 'table', label: copy.tableView },
  ]);

  /** Oldest first, so stepping left is stepping back in time. */
  const orderedPeriods = $derived(
    [...periods].sort((left, right) => periodStartYear(left.label) - periodStartYear(right.label))
  );

  const selectedYearIndex = $derived(
    orderedPeriods.findIndex((period) => period.id === selectedPeriod?.id)
  );

  function stepYear(delta: number) {
    const target = orderedPeriods[selectedYearIndex + delta];
    if (target) selectedPeriodId = target.id;
  }

  function statusLabel(status: 'excused' | 'unexcused' | 'pending'): string {
    if (status === 'excused') return copy.entryExcused;
    return status === 'unexcused' ? copy.entryUnexcused : copy.entryPending;
  }

  function statusTone(status: 'excused' | 'unexcused' | 'pending') {
    if (status === 'excused') return 'success' as const;
    return status === 'unexcused' ? ('danger' as const) : ('warning' as const);
  }

  /** One row per missed session, newest first, block named on every row. */
  const tableRows = $derived.by(() => {
    locale;
    return blocks.flatMap((block) => {
      const { title } = splitBlockLabel(block.label);
      return sortEntriesByDateDesc(block.entries).map((entry) => ({
        id: entry.id,
        block: title,
        date: entry.time ? `${entry.date} · ${entry.time}` : entry.date,
        course: entry.course,
        duration: formatDuration(entry.duration),
        status: absenceStatus(entry),
        reason: entry.reason ?? '-',
      }));
    });
  });

  function blockReport(block: AbsenceBlock): PortalDocument | null {
    if (!block.reportPath) return null;
    return {
      kind: 'absenceReport',
      label: `${copy.report} - ${splitBlockLabel(block.label).title}`,
      requestPath: block.reportPath,
      suggestedFilename: null,
    };
  }

  async function downloadBlockDocument(document: PortalDocument) {
    downloadingPath = document.requestPath;
    downloadFailed = false;
    try {
      await downloadPortalDocument(document);
    } catch {
      downloadFailed = true;
    } finally {
      downloadingPath = null;
    }
  }
</script>

<PageShell>
  {#if absencesState.kind === 'loading'}
    <AbsencesViewSkeleton ariaLabel={copy.loading} heroLabel={copy.totalHours} />
  {:else if absencesState.kind === 'error'}
    {@const expired = absencesState.code === 'session_expired'}
    {#if !connectivity.online}
      <!-- A device without a network path is not a portal outage. -->
      <StateCard
        kind="error"
        icon={CloudOff}
        title={copy.offlineHeading}
        description={copy.offlineDescription}
        actionLabel={copy.retry}
        onAction={() => loadAbsences(true)}
      />
    {:else}
      <StateCard
        kind={expired ? 'expired' : 'error'}
        icon={AlertCircle}
        title={copy.errorHeading}
        description={resourceErrorMessage(absencesState.code)}
        actionLabel={expired ? copy.backToLogin : copy.retry}
        onAction={expired ? () => void onLogout() : () => loadAbsences(true)}
      />
    {/if}
  {:else}
    {#if periods.length === 0}
      <StateCard
        kind="empty"
        icon={CalendarX2}
        title={copy.emptyHeading}
        description={copy.emptyDescription}
        actionLabel={copy.refresh}
        onAction={() => loadAbsences(true)}
      />
    {:else}
      <!-- The year is always named and always steppable, so an older year is one
           click away and the current one is never guessed from the sessions. -->
      <div class="year-switch">
        <IconButton
          label={copy.previousYear}
          variant="ghost"
          disabled={selectedYearIndex <= 0}
          onclick={() => stepYear(-1)}
        >
          <ChevronLeft size={17} aria-hidden="true" />
        </IconButton>

        <p class="year-current">
          <span>{copy.yearLabel}</span>
          <strong>{selectedPeriod?.label ?? '--'}</strong>
        </p>

        <IconButton
          label={copy.nextYear}
          variant="ghost"
          disabled={selectedYearIndex < 0 || selectedYearIndex >= orderedPeriods.length - 1}
          onclick={() => stepYear(1)}
        >
          <ChevronRight size={17} aria-hidden="true" />
        </IconButton>
      </div>

      <HeroStat
        ariaLabel={copy.heading}
        label={copy.totalHours}
        value={formatHours(totals.hours)}
        unit="h"
        showCurve={false}
      >
        {#snippet metrics()}
          <HeroMetric
            title={copy.presenceRate}
            value={`${formatHours(presencePercentage)}%`}
            tone={presencePercentage >= 90 ? 'success' : presencePercentage >= 70 ? 'warning' : 'danger'}
          >
            {#snippet icon()}<ShieldCheck size={13} aria-hidden="true" />{/snippet}
          </HeroMetric>

          <HeroMetric
            title={copy.sessionCount}
            value={String(totals.count)}
            tone="neutral"
          >
            {#snippet icon()}<CalendarX2 size={13} aria-hidden="true" />{/snippet}
          </HeroMetric>

          <HeroMetric
            title={copy.excused}
            value={formatHours(totals.excusedHours)}
            unit="h"
            tone={totals.excusedHours > 0 ? 'success' : 'neutral'}
          >
            {#snippet icon()}<CheckCircle2 size={13} aria-hidden="true" />{/snippet}
          </HeroMetric>

          <HeroMetric
            title={copy.unexcused}
            value={formatHours(totals.unexcusedHours)}
            unit="h"
            tone={totals.unexcusedHours > 0 ? 'danger' : 'neutral'}
          >
            {#snippet icon()}<XCircle size={13} aria-hidden="true" />{/snippet}
          </HeroMetric>

          {#if totals.pendingHours > 0}
            <HeroMetric
              title={copy.pending}
              value={formatHours(totals.pendingHours)}
              unit="h"
              tone="warning"
            >
              {#snippet icon()}<Clock size={13} aria-hidden="true" />{/snippet}
            </HeroMetric>
          {/if}
        {/snippet}
      </HeroStat>

      {#if downloadFailed}
        <p class="download-error" role="alert">{copy.downloadFailed}</p>
      {/if}

      <div class="view-toolbar">
        <div class="toolbar-actions">
          {#if periodEntryList.length > 0}
            <SegmentedControl
              options={viewOptions}
              value={viewMode}
              size="sm"
              label={copy.viewModeLabel}
              onChange={(value) => (viewMode = value as 'cards' | 'table')}
            />
          {/if}

          <div class="desktop-only">
            <IconButton
              label={copy.refresh}
              variant="ghost"
              size="sm"
              loading={refreshing}
              onclick={() => loadAbsences(true)}
            >
              <RefreshCw size={14} aria-hidden="true" />
            </IconButton>
          </div>
        </div>
      </div>

      {#if periodEntryList.length === 0}
        <StateCard
          kind="empty"
          icon={PartyPopper}
          title={copy.emptyTitle}
          description={copy.emptyYearDescription}
        />
      {/if}

      {#if viewMode === 'cards'}
        <div class="block-list">
          {#each blocks as block (block.id)}
            {@const label = splitBlockLabel(block.label)}
            {@const entries = sortEntriesByDateDesc(block.entries)}
            {@const open = openBlockIds.includes(block.id)}
            {@const report = blockReport(block)}
            <section class="block-card" class:is-open={open}>
              <h2 class="block-heading">
                <button
                  type="button"
                  class="block-toggle"
                  aria-expanded={open}
                  aria-controls={`absence-block-panel-${block.id}`}
                  onclick={() => toggleBlock(block.id)}
                >
                  <span class="block-chevron" class:is-open={open}>
                    <ChevronDown size={18} aria-hidden="true" />
                  </span>
                  <span class="block-identity">
                    <span class="block-title">{label.title}</span>
                    {#if label.range}<small class="block-range">{label.range}</small>{/if}
                  </span>
                  <span class="block-hours" title={copy.blockHours}>
                    <strong>{formatHours(blockHours(block))}</strong>
                    <small>h</small>
                  </span>
                </button>
              </h2>

              <div id={`absence-block-panel-${block.id}`} class="block-panel" hidden={!open}>
                {#if report}
                  <div class="block-documents">
                    <button
                      type="button"
                      class="document-button"
                      disabled={downloadingPath === report.requestPath}
                      onclick={() => downloadBlockDocument(report)}
                    >
                      {#if downloadingPath === report.requestPath}
                        <Spinner size={14} />
                      {:else}
                        <FileText size={14} aria-hidden="true" />
                      {/if}
                      {copy.report}
                    </button>
                  </div>
                {/if}

                {#if entries.length === 0}
                  <p class="block-empty">{copy.blockEmpty}</p>
                {:else}
                  <div class="absence-timeline">
                    {#each entries as entry (entry.id)}
                      {@const status = absenceStatus(entry)}
                      <article class="timeline-item">
                        <div class="item-datetime">
                          <strong>{entry.date}</strong>
                          {#if entry.time}
                            <span>{entry.time}</span>
                          {/if}
                        </div>

                        <div class="item-card" class:is-excused={status === 'excused'} class:is-unexcused={status === 'unexcused'}>
                          <div class="card-topline">
                            <span class="duration-chip">
                              <Clock size={12} aria-hidden="true" />
                              {formatDuration(entry.duration)}
                            </span>

                            <Badge tone={statusTone(status)}>
                              {#if status === 'excused'}
                                <CheckCircle2 size={12} aria-hidden="true" />
                              {:else if status === 'unexcused'}
                                <XCircle size={12} aria-hidden="true" />
                              {:else}
                                <Clock size={12} aria-hidden="true" />
                              {/if}
                              {statusLabel(status)}
                            </Badge>
                          </div>

                          <h3 class="course-name">{entry.course}</h3>

                          {#if entry.reason}
                            <p class="reason-row">
                              <span class="reason-label">{copy.reasonLabel}</span>
                              <span class="reason-text">{entry.reason}</span>
                            </p>
                          {/if}
                        </div>
                      </article>
                    {/each}
                  </div>
                {/if}
              </div>
            </section>
          {/each}
        </div>
      {:else}
        <section class="table-container">
          <div class="table-scroll">
            <table class="data-table">
              <thead>
                <tr>
                  <th scope="col">{copy.tableBlock}</th>
                  <th scope="col">{copy.tableDate}</th>
                  <th scope="col">{copy.tableCourse}</th>
                  <th scope="col" class="text-right">{copy.tableDuration}</th>
                  <th scope="col">{copy.tableStatus}</th>
                  <th scope="col">{copy.tableReason}</th>
                </tr>
              </thead>
              <tbody>
                {#each tableRows as row (row.id)}
                  <tr>
                    <td class="table-secondary-cell">
                      <span class="cell-label">{copy.tableBlock}</span>
                      <span>{row.block}</span>
                    </td>
                    <td>
                      <span class="cell-label">{copy.tableDate}</span>
                      <span class="table-date-cell">{row.date}</span>
                    </td>
                    <td>
                      <span class="cell-label">{copy.tableCourse}</span>
                      <strong>{row.course}</strong>
                    </td>
                    <td class="text-right">
                      <span class="cell-label">{copy.tableDuration}</span>
                      <span class="table-duration-badge">{row.duration}</span>
                    </td>
                    <td>
                      <span class="cell-label">{copy.tableStatus}</span>
                      <Badge tone={statusTone(row.status)}>{statusLabel(row.status)}</Badge>
                    </td>
                    <td class="table-secondary-cell">
                      <span class="cell-label">{copy.tableReason}</span>
                      <span>{row.reason}</span>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </section>
      {/if}
    {/if}
  {/if}
</PageShell>

<style>
  /* Toolbar */
  .view-toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-3);
  }

  .toolbar-actions {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: var(--space-2);
  }

  .year-switch {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    width: 100%;
  }

  .year-current {
    display: flex;
    flex: 1;
    justify-content: center;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--space-2);
    margin: 0;
  }

  .year-current span {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .year-current strong {
    color: var(--foreground);
    font-size: var(--text-base);
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-bold);
  }

  .download-error {
    margin: 0;
    padding: var(--space-2) var(--space-3);
    color: var(--danger-strong);
    background: var(--danger-surface);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
  }

  /* Year → block accordion, mirroring how the portal groups the absences. */
  .block-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .block-card {
    overflow: hidden;
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
  }

  .block-card.is-open {
    border-color: var(--border);
  }

  .block-heading {
    margin: 0;
    font-size: inherit;
    font-weight: inherit;
  }

  .block-toggle {
    display: flex;
    width: 100%;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-4);
    background: transparent;
    border: 0;
    cursor: pointer;
    text-align: left;
  }

  .block-chevron {
    display: inline-flex;
    flex-shrink: 0;
    color: var(--muted-foreground);
    transition: transform var(--duration-fast, 150ms) ease;
  }

  .block-chevron.is-open {
    transform: rotate(180deg);
  }

  .block-identity {
    display: flex;
    min-width: 0;
    flex: 1;
    flex-direction: column;
    gap: 0.15rem;
  }

  .block-title {
    color: var(--foreground);
    font-size: var(--text-base);
    font-weight: var(--weight-bold);
    line-height: 1.3;
  }

  .block-range {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .block-hours {
    display: inline-flex;
    flex-shrink: 0;
    align-items: baseline;
    gap: 0.1rem;
  }

  .block-hours strong {
    color: var(--primary-deep);
    font-size: var(--text-xl);
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-heavy);
  }

  .block-hours small {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
  }

  .block-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    /* The header keeps its own padding, so the panel only re-opens the gap the
       separator needs. */
    margin: 0 var(--space-4);
    padding: var(--space-4) 0;
    border-top: 1px solid var(--border-subtle);
  }

  .block-panel[hidden] {
    display: none;
  }

  .block-documents {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .document-button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0.35rem var(--space-3);
    color: var(--foreground);
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-pill);
    cursor: pointer;
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
  }

  .document-button:disabled {
    cursor: progress;
    opacity: 0.7;
  }

  .block-empty {
    margin: 0;
    color: var(--muted-foreground);
    font-size: var(--text-sm);
  }

  /* Timeline */
  .absence-timeline {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .timeline-item {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-1);
    align-items: stretch;
  }

  .item-datetime {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
  }

  .item-datetime strong {
    color: var(--foreground);
    font-weight: var(--weight-bold);
  }

  .item-datetime span {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
  }

  .item-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
  }

  /* The tint repeats what the status badge already says in words. */
  .item-card.is-excused {
    background: var(--success-surface);
    border-color: color-mix(in oklch, var(--success) 30%, transparent);
  }

  .item-card.is-unexcused {
    background: var(--danger-surface);
    border-color: color-mix(in oklch, var(--danger) 30%, transparent);
  }

  .card-topline {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .duration-chip {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 0.2rem var(--space-2);
    color: var(--muted-foreground);
    background: var(--card);
    border-radius: var(--radius-sm);
    font-size: var(--text-2xs);
    font-weight: var(--weight-bold);
    font-variant-numeric: tabular-nums;
  }

  .course-name {
    margin: 0;
    color: var(--foreground);
    font-size: var(--text-md);
    font-weight: var(--weight-bold);
    line-height: 1.3;
  }

  .reason-row {
    display: flex;
    align-items: baseline;
    gap: var(--space-1);
    margin: 0;
    font-size: var(--text-sm);
  }

  .reason-label {
    color: var(--muted-foreground);
    font-weight: var(--weight-semibold);
  }

  .reason-text {
    color: var(--foreground);
    font-weight: var(--weight-medium);
  }

  /* Table view. Base layout is the small-screen card fallback; the real table
     only assembles once there is room for six columns. */
  .table-container {
    overflow: hidden;
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
  }

  .table-scroll {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }

  .data-table {
    display: block;
    width: 100%;
    padding: var(--space-3);
    border-collapse: collapse;
    font-size: var(--text-base);
    text-align: left;
  }

  .data-table thead {
    display: none;
  }

  .data-table tbody,
  .data-table tr,
  .data-table td {
    display: block;
  }

  .data-table tr + tr {
    margin-top: var(--space-2);
  }

  .data-table tr {
    padding: var(--space-3);
    background: var(--surface-sunken);
    border-radius: var(--radius-md);
  }

  .data-table td {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-2);
    padding: var(--space-1) 0;
  }

  .cell-label {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
  }

  .table-date-cell {
    font-variant-numeric: tabular-nums;
  }

  .table-duration-badge {
    display: inline-block;
    padding: 0.2rem var(--space-2);
    color: var(--primary-deep);
    background: var(--muted);
    border-radius: var(--radius-sm);
    font-weight: var(--weight-bold);
    font-variant-numeric: tabular-nums;
  }

  .table-secondary-cell {
    color: var(--muted-foreground);
    font-size: var(--text-sm);
  }

  @media (min-width: 48rem) {
    .year-switch {
      width: auto;
    }

    .timeline-item {
      grid-template-columns: 7rem minmax(0, 1fr);
      gap: var(--space-4);
    }

    .item-datetime {
      flex-direction: column;
      align-items: flex-start;
      justify-content: center;
      gap: var(--space-1);
    }

    .data-table {
      display: table;
      padding: 0;
    }

    .data-table thead {
      display: table-header-group;
    }

    .data-table tbody {
      display: table-row-group;
    }

    .data-table tr,
    .data-table tr + tr {
      display: table-row;
      margin: 0;
      padding: 0;
      background: transparent;
      border-radius: 0;
    }

    .data-table td {
      display: table-cell;
      padding: var(--space-3) var(--space-4);
      border-bottom: 1px solid var(--border-subtle);
      vertical-align: middle;
    }

    .data-table th {
      padding: var(--space-3) var(--space-4);
      color: var(--muted-foreground);
      background: var(--surface-sunken);
      border-bottom: 1px solid var(--border-subtle);
      font-size: var(--text-xs);
      font-weight: var(--weight-bold);
      white-space: nowrap;
    }

    .data-table tbody tr:last-child td {
      border-bottom: 0;
    }

    .cell-label {
      display: none;
    }

    .text-right {
      text-align: right;
    }
  }

  @media (hover: hover) {
    .block-toggle:hover {
      background: var(--surface-sunken);
    }

    .document-button:hover:not(:disabled) {
      background: var(--muted);
    }

    .data-table tbody tr:hover td {
      background: var(--surface-sunken);
    }
  }
</style>
