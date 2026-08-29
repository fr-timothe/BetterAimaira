<script lang="ts">
  import { onMount } from 'svelte';
  import {
    CalendarX2,
    CheckCircle2,
    Clock,
    FileText,
    PartyPopper,
    ShieldCheck,
    XCircle,
  } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import DataTable, { cellLabel } from '$lib/components/ui/DataTable.svelte';
  import PageShell from '$lib/components/ui/PageShell.svelte';
  import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import HeroMetric from './HeroMetric.svelte';
  import HeroStat from './HeroStat.svelte';
  import AbsencesViewSkeleton from './AbsencesViewSkeleton.svelte';
  import BlockAccordion from './BlockAccordion.svelte';
  import DocumentPill from './DocumentPill.svelte';
  import PortalResourceView from './PortalResourceView.svelte';
  import YearStepper from './YearStepper.svelte';
  import {
    absenceStatus,
    absenceTotals,
    blockHours,
    calculatePresenceRate,
    periodEntries,
    sortEntriesByDateDesc,
  } from './absence-utils';
  import {
    createBlockDisclosure,
    createDocumentDownload,
    latestPeriod,
  } from './academic-view.svelte';
  import { createPortalResource } from './portal-resource.svelte';
  import { splitBlockLabel } from './portal-utils';
  import type { AbsenceBlock, PortalDocument } from './types';
  import { cn } from '$lib/utils';

  type Props = {
    locale: Locale;
    onLogout: () => Promise<void>;
    refresh?: () => Promise<void>;
  };

  let { locale, onLogout, refresh = $bindable() }: Props = $props();

  let viewMode = $state<'cards' | 'table'>('cards');
  let selectedPeriodId = $state<string | null>(null);

  const copy = $derived.by(() => {
    return {
      heading: m.absences_heading(),
      totalHours: m.absence_total_hours(),
      presenceRate: m.absence_presence_rate(),
      sessionCount: m.absence_session_count(),
      excused: m.absence_excused(),
      unexcused: m.absence_unexcused(),
      pending: m.absence_pending(),
      loading: m.resource_loading(),
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

  const absences = createPortalResource({
    resource: 'absences',
    fallbackErrorCode: 'absences_unavailable',
    heading: () => copy.heading,
    locale: () => locale,
  });

  const disclosure = createBlockDisclosure();
  const downloads = createDocumentDownload();

  $effect(() => {
    refresh = () => absences.load(true);
  });

  onMount(() => {
    void absences.load();
  });

  const periods = $derived(absences.page?.absencePeriods ?? []);

  const selectedPeriod = $derived(
    periods.find((period) => period.id === selectedPeriodId) ?? latestPeriod(periods)
  );

  const blocks = $derived(selectedPeriod?.blocks ?? []);

  const periodEntryList = $derived(selectedPeriod ? periodEntries(selectedPeriod) : []);

  const totals = $derived(absenceTotals(periodEntryList));
  const presencePercentage = $derived(calculatePresenceRate(totals, blocks.length));

  const openBlockIds = $derived(disclosure.openIds(selectedPeriod?.id));

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

  // The tint repeats what the status badge already says in words.
  const cardTones = {
    excused: 'border-success-edge bg-success-surface',
    unexcused: 'border-danger-edge bg-danger-surface',
    pending: 'border-border-subtle bg-surface-sunken'
  } as const;
</script>

<PageShell>
  <PortalResourceView
    resource={absences}
    {locale}
    {onLogout}
    empty={periods.length === 0}
    emptyIcon={CalendarX2}
  >
    {#snippet skeleton()}
      <AbsencesViewSkeleton ariaLabel={copy.loading} heroLabel={copy.totalHours} />
    {/snippet}

    {#snippet controls()}
      {#if periodEntryList.length > 0}
        <SegmentedControl
          options={viewOptions}
          value={viewMode}
          size="sm"
          label={copy.viewModeLabel}
          onChange={(value) => (viewMode = value as 'cards' | 'table')}
        />
      {/if}
    {/snippet}

    {#snippet ready(toolbar)}
      <YearStepper
        {periods}
        selectedId={selectedPeriod?.id ?? null}
        onSelect={(id) => (selectedPeriodId = id)}
        label={copy.yearLabel}
        previousLabel={copy.previousYear}
        nextLabel={copy.nextYear}
      />

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

      {#if downloads.failed}
        <p
          class="rounded-md bg-danger-surface px-3 py-2 text-sm font-semibold text-danger-strong"
          role="alert"
        >{copy.downloadFailed}</p>
      {/if}

      {@render toolbar()}

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
            {@const report = blockReport(block)}
            <BlockAccordion
              panelId={`absence-block-panel-${block.id}`}
              title={label.title}
              range={label.range}
              value={formatHours(blockHours(block))}
              unit="h"
              valueTitle={copy.blockHours}
              open={openBlockIds.includes(block.id)}
              onToggle={() => disclosure.toggle(selectedPeriod?.id, block.id)}
            >
              {#if report}
                <div class="flex flex-wrap gap-2">
                  <DocumentPill
                    label={copy.report}
                    icon={FileText}
                    busy={downloads.requestPath === report.requestPath}
                    onclick={() => downloads.download(report)}
                  />
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
            </BlockAccordion>
          {/each}
        </div>
      {:else}
        <DataTable>
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
        </DataTable>
      {/if}
    {/snippet}
  </PortalResourceView>
</PageShell>
