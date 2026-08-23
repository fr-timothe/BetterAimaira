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
  import { cn } from '$lib/utils';

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
  const cellLabel = 'cell-label text-xs font-bold text-muted-foreground';

  // The tint repeats what the status badge already says in words.
  const cardTones = {
    excused: 'border-success-edge bg-success-surface',
    unexcused: 'border-danger-edge bg-danger-surface',
    pending: 'border-border-subtle bg-surface-sunken'
  } as const;
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
      <div class="flex w-full items-center justify-between gap-2 md:w-auto">
        <IconButton
          label={copy.previousYear}
          variant="ghost"
          disabled={selectedYearIndex <= 0}
          onclick={() => stepYear(-1)}
        >
          <ChevronLeft size={17} aria-hidden="true" />
        </IconButton>

        <p class="flex flex-1 flex-wrap items-baseline justify-center gap-2">
          <span class="text-xs font-bold tracking-[0.04em] uppercase text-muted-foreground"
            >{copy.yearLabel}</span
          >
          <strong class="text-base font-bold tabular-nums text-foreground"
            >{selectedPeriod?.label ?? '--'}</strong
          >
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
        <p
          class="rounded-md bg-danger-surface px-3 py-2 text-sm font-semibold text-danger-strong"
          role="alert"
        >{copy.downloadFailed}</p>
      {/if}

      <div class="flex flex-wrap items-center justify-end gap-3">
        <div class="flex min-w-0 items-center gap-2">
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
        <!-- Year to block accordion, mirroring how the portal groups absences. -->
        <div class="flex flex-col gap-3">
          {#each blocks as block (block.id)}
            {@const label = splitBlockLabel(block.label)}
            {@const entries = sortEntriesByDateDesc(block.entries)}
            {@const open = openBlockIds.includes(block.id)}
            {@const report = blockReport(block)}
            <section
              class={cn(
                'overflow-hidden rounded-xl border bg-card',
                open ? 'border-border' : 'border-border-subtle'
              )}
            >
              <h2>
                <button
                  type="button"
                  class="flex w-full cursor-pointer items-center gap-3 bg-transparent p-4
                         text-left hover:bg-surface-sunken"
                  aria-expanded={open}
                  aria-controls={`absence-block-panel-${block.id}`}
                  onclick={() => toggleBlock(block.id)}
                >
                  <span
                    class={cn(
                      'inline-flex shrink-0 text-muted-foreground transition-transform',
                      'duration-fast ease-[ease]',
                      open && 'rotate-180'
                    )}
                  >
                    <ChevronDown size={18} aria-hidden="true" />
                  </span>
                  <span class="flex min-w-0 flex-1 flex-col gap-[0.15rem]">
                    <span class="text-base leading-[1.3] font-bold text-foreground"
                      >{label.title}</span
                    >
                    {#if label.range}
                      <small class="text-xs tabular-nums text-muted-foreground">{label.range}</small>
                    {/if}
                  </span>
                  <span class="inline-flex shrink-0 items-baseline gap-[0.1rem]" title={copy.blockHours}>
                    <strong class="text-xl font-extrabold tabular-nums text-primary-deep"
                      >{formatHours(blockHours(block))}</strong
                    >
                    <small class="text-xs font-semibold text-muted-foreground">h</small>
                  </span>
                </button>
              </h2>

              <!-- The header keeps its own padding, so the panel only re-opens the
                   gap the separator needs. -->
              <div
                id={`absence-block-panel-${block.id}`}
                class="mx-4 flex flex-col gap-4 border-t border-border-subtle py-4
                       [&[hidden]]:hidden"
                hidden={!open}
              >
                {#if report}
                  <div class="flex flex-wrap gap-2">
                    <button
                      type="button"
                      class="inline-flex cursor-pointer items-center gap-2 rounded-pill border
                             border-border-subtle bg-surface-sunken px-3 py-[0.35rem] text-xs
                             font-semibold text-foreground disabled:cursor-progress
                             disabled:opacity-70 enabled:hover:bg-muted"
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
                  <p class="text-sm text-muted-foreground">{copy.blockEmpty}</p>
                {:else}
                  <div class="flex flex-col gap-3">
                    {#each entries as entry (entry.id)}
                      {@const status = absenceStatus(entry)}
                      <article
                        class="grid grid-cols-[minmax(0,1fr)] items-stretch gap-1
                               md:grid-cols-[7rem_minmax(0,1fr)] md:gap-4"
                      >
                        <div
                          class="flex items-baseline gap-2 text-sm tabular-nums md:flex-col
                                 md:items-start md:justify-center md:gap-1"
                        >
                          <strong class="font-bold text-foreground">{entry.date}</strong>
                          {#if entry.time}
                            <span class="text-xs text-muted-foreground">{entry.time}</span>
                          {/if}
                        </div>

                        <div
                          class={cn(
                            'flex flex-col gap-2 rounded-lg border px-4 py-3',
                            cardTones[status] ?? cardTones.pending
                          )}
                        >
                          <div class="flex items-center justify-between gap-2">
                            <span
                              class="inline-flex items-center gap-1 rounded-sm bg-card px-2
                                     py-[0.2rem] text-2xs font-bold tabular-nums
                                     text-muted-foreground"
                            >
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

                          <h3 class="text-md leading-[1.3] font-bold text-foreground"
                            >{entry.course}</h3
                          >

                          {#if entry.reason}
                            <p class="flex items-baseline gap-1 text-sm">
                              <span class="font-semibold text-muted-foreground"
                                >{copy.reasonLabel}</span
                              >
                              <span class="font-medium text-foreground">{entry.reason}</span>
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
        <section class="overflow-hidden rounded-xl border border-border-subtle bg-card">
          <div class="overflow-x-auto [-webkit-overflow-scrolling:touch]">
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
                    <td class="text-sm text-muted-foreground">
                      <span class={cellLabel}>{copy.tableBlock}</span>
                      <span>{row.block}</span>
                    </td>
                    <td>
                      <span class={cellLabel}>{copy.tableDate}</span>
                      <span class="tabular-nums">{row.date}</span>
                    </td>
                    <td>
                      <span class={cellLabel}>{copy.tableCourse}</span>
                      <strong>{row.course}</strong>
                    </td>
                    <td class="text-right">
                      <span class={cellLabel}>{copy.tableDuration}</span>
                      <span class="inline-block rounded-sm bg-muted px-2 py-[0.2rem] font-bold tabular-nums text-primary-deep">{row.duration}</span>
                    </td>
                    <td>
                      <span class={cellLabel}>{copy.tableStatus}</span>
                      <Badge tone={statusTone(row.status)}>{statusLabel(row.status)}</Badge>
                    </td>
                    <td class="text-sm text-muted-foreground">
                      <span class={cellLabel}>{copy.tableReason}</span>
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
  /* Table view. The base layout is the small-screen card fallback; the real
     table only assembles once there is room for six columns. Switching a
     table's own display model means rewriting table, thead, tbody, tr, td and
     th together, so it stays one block of CSS rather than a class on every
     cell — and `.cell-label` belongs to that mechanism. */
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

  @media (min-width: 48rem) {
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
  }

  @media (hover: hover) {
    .data-table tbody tr:hover td {
      background: var(--surface-sunken);
    }
  }
</style>
