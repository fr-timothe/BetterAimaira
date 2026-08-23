<script lang="ts">
  import { onMount } from 'svelte';
  import {
    AlertCircle,
    BookOpenCheck,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    CloudOff,
    Download,
    FileText,
    RefreshCw,
    Sparkles,
    TrendingDown,
    TrendingUp,
  } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import PageShell from '$lib/components/ui/PageShell.svelte';
  import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import { connectivity } from '$lib/state/connectivity.svelte';
  import HeroMetric from './HeroMetric.svelte';
  import HeroStat from './HeroStat.svelte';
  import AcademicViewSkeleton from './AcademicViewSkeleton.svelte';
  import { loadPortalResource } from './portal-cache';
  import {
    averageOfCourses,
    blockCourses,
    courseAverage,
    cumulativeAverages,
    evaluationScore,
    extremeScaledValue,
    gradedEvaluations,
    periodCourses,
  } from './grade-utils';
  import {
    downloadPortalDocument,
    getSubjectColor,
    parseResourceError,
    periodStartYear,
    resourceErrorMessage,
    splitBlockLabel,
  } from './portal-utils';
  import type {
    GradeBlock,
    GradeEvaluation,
    PortalDocument,
    PortalDocumentKind,
    PortalResourceState,
  } from './types';

  type Props = {
    locale: Locale;
    onLogout: () => Promise<void>;
    refresh?: () => Promise<void>;
  };

  let { locale, onLogout, refresh = $bindable() }: Props = $props();

  $effect(() => {
    refresh = () => loadGrades(true);
  });

  let gradesState = $state<PortalResourceState>({ kind: 'loading' });
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
      heading: m.grades_heading(),
      average: m.grade_average(),
      count: m.grade_count(),
      highest: m.grade_highest(),
      lowest: m.grade_lowest(),
      cardsView: m.cards_view(),
      tableView: m.table_view(),
      loading: m.resource_loading(),
      refresh: m.resource_refresh(),
      errorHeading: m.resource_error_heading(),
      retry: m.resource_retry(),
      backToLogin: m.back_to_login(),
      emptyHeading: m.resource_empty_heading(),
      emptyDescription: m.resource_empty_description(),
      offlineHeading: m.sync_offline(),
      offlineDescription: m.sync_offline_description(),
      viewModeLabel: m.grade_view_mode_label(),
      yearLabel: m.grade_year_label(),
      previousYear: m.grade_previous_year(),
      nextYear: m.grade_next_year(),
      trendLabel: m.grade_trend_label(),
      blockAverage: m.grade_block_average(),
      blockEmpty: m.grade_block_empty(),
      noGradeYet: m.grade_no_grade_yet(),
      bulletin: m.document_kind_grade_bulletin(),
      transcript: m.document_kind_grade_transcript(),
      downloadFailed: m.grade_download_failed(),
      tableBlock: m.grade_table_block(),
      tableCourse: m.grade_table_course(),
      tableEvaluation: m.grade_table_evaluation(),
      tableGrade: m.grade_table_grade(),
      tableWeight: m.grade_table_weight(),
    };
  });

  onMount(() => {
    void loadGrades();
  });

  async function loadGrades(force = false) {
    const sequence = ++requestSequence;
    const hasData = gradesState.kind === 'ready';
    if (hasData) refreshing = true;
    else gradesState = { kind: 'loading' };
    try {
      const page = await loadPortalResource('grades', force);
      if (sequence !== requestSequence) return;
      gradesState = { kind: 'ready', page };
      refreshFailed = false;
    } catch (error) {
      if (sequence !== requestSequence) return;
      // Keeping stale data on screen is fine; keeping it unmarked is not. Flag
      // the failure so the freshness label can say what the user is looking at.
      if (hasData) refreshFailed = true;
      else gradesState = { kind: 'error', code: parseResourceError(error, 'grades_unavailable') };
    } finally {
      if (sequence === requestSequence) refreshing = false;
    }
  }

  const fetchedAt = $derived(gradesState.kind === 'ready' ? gradesState.page.fetchedAt : null);

  const periods = $derived(gradesState.kind === 'ready' ? gradesState.page.gradePeriods : []);

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

  const periodCourseList = $derived(selectedPeriod ? periodCourses(selectedPeriod) : []);

  const overallAverage = $derived(averageOfCourses(periodCourseList));
  const highestGrade = $derived(extremeScaledValue(periodCourseList, 'highest'));
  const lowestGrade = $derived(extremeScaledValue(periodCourseList, 'lowest'));
  const evaluationCount = $derived(gradedEvaluations(periodCourseList).length);
  const trendSeries = $derived(cumulativeAverages(periodCourseList));

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

  function formatGrade(value: number | null | undefined): string {
    if (value === null || value === undefined) return '--';
    return value.toLocaleString(locale, { minimumFractionDigits: 1, maximumFractionDigits: 2 });
  }

  function formatEvaluationScore(evaluation: GradeEvaluation): string {
    const { value, max } = evaluationScore(evaluation);
    if (value === null) return evaluation.score ?? '--';
    // The scale is a whole number on this portal: no forced decimal on it.
    const scale = max.toLocaleString(locale, { maximumFractionDigits: 2 });
    return `${formatGrade(value)}/${scale}`;
  }

  /**
   * Returns a tone, not a colour: the badge renders it through the shared token
   * pairs so the same standing looks the same everywhere. The label always
   * states the standing too.
   */
  function getGradeStatusBadge(average: number | null) {
    if (average === null) return { label: m.grade_mention_pending(), tone: 'neutral' as const };
    if (average >= 16) return { label: m.grade_mention_excellent(), tone: 'success' as const };
    if (average >= 14) return { label: m.grade_mention_good(), tone: 'accent' as const };
    if (average >= 12) return { label: m.grade_mention_satisfactory(), tone: 'accent' as const };
    if (average >= 10) return { label: m.grade_mention_sufficient(), tone: 'warning' as const };
    return { label: m.grade_mention_struggling(), tone: 'danger' as const };
  }

  const statusBadge = $derived.by(() => {
    locale;
    return getGradeStatusBadge(overallAverage);
  });

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

  /** One row per evaluation, sub-evaluations included and marked as such. */
  const tableRows = $derived.by(() => {
    locale;
    const rows: {
      id: string;
      block: string;
      course: string;
      label: string;
      score: string;
      weight: string;
      isChild: boolean;
    }[] = [];

    for (const block of blocks) {
      const { title } = splitBlockLabel(block.label);
      for (const course of blockCourses(block)) {
        const push = (evaluation: GradeEvaluation, isChild: boolean, key: string) => {
          rows.push({
            id: key,
            block: title,
            course: course.name,
            label: evaluation.label || course.name,
            score: formatEvaluationScore(evaluation),
            weight: evaluation.weight ?? '-',
            isChild,
          });
          evaluation.children.forEach((child, childIndex) =>
            push(child, true, `${key}-${childIndex}`)
          );
        };
        course.evaluations.forEach((evaluation, index) =>
          push(evaluation, false, `${course.id}-${index}`)
        );
      }
    }

    return rows;
  });

  function blockDocument(
    block: GradeBlock,
    kind: Extract<PortalDocumentKind, 'gradeBulletin' | 'gradeTranscript'>
  ): PortalDocument | null {
    const requestPath = kind === 'gradeBulletin' ? block.bulletinPath : block.transcriptPath;
    if (!requestPath) return null;
    const label = kind === 'gradeBulletin' ? copy.bulletin : copy.transcript;
    return {
      kind,
      label: `${label} - ${splitBlockLabel(block.label).title}`,
      requestPath,
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
  {#if gradesState.kind === 'loading'}
    <AcademicViewSkeleton ariaLabel={copy.loading} heroLabel={copy.average} />
  {:else if gradesState.kind === 'error'}
    {@const expired = gradesState.code === 'session_expired'}
    {#if !connectivity.online}
      <!-- A device without a network path is not a portal outage. -->
      <StateCard
        kind="error"
        icon={CloudOff}
        title={copy.offlineHeading}
        description={copy.offlineDescription}
        actionLabel={copy.retry}
        onAction={() => loadGrades(true)}
      />
    {:else}
      <StateCard
        kind={expired ? 'expired' : 'error'}
        icon={AlertCircle}
        title={copy.errorHeading}
        description={resourceErrorMessage(gradesState.code)}
        actionLabel={expired ? copy.backToLogin : copy.retry}
        onAction={expired ? () => void onLogout() : () => loadGrades(true)}
      />
    {/if}
  {:else}
    {#if periods.length === 0}
      <StateCard
        kind="empty"
        icon={BookOpenCheck}
        title={copy.emptyHeading}
        description={copy.emptyDescription}
        actionLabel={copy.refresh}
        onAction={() => loadGrades(true)}
      />
    {:else}
      <!-- The year is always named and always steppable, so an older year is one
           click away and the current one is never guessed from the marks. -->
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
        label={copy.average}
        value={formatGrade(overallAverage)}
        unit="/20"
        trend={trendSeries}
        trendLabel={copy.trendLabel}
      >
        {#snippet badge()}
          <Badge tone={statusBadge.tone}>
            <Sparkles size={13} aria-hidden="true" />
            {statusBadge.label}
          </Badge>
        {/snippet}

        {#snippet metrics()}
          <HeroMetric title={copy.count} value={String(evaluationCount)} />

          <HeroMetric title={copy.highest} value={formatGrade(highestGrade)} unit="/20">
            {#snippet icon()}<TrendingUp size={13} aria-hidden="true" />{/snippet}
          </HeroMetric>

          <HeroMetric title={copy.lowest} value={formatGrade(lowestGrade)} unit="/20">
            {#snippet icon()}<TrendingDown size={13} aria-hidden="true" />{/snippet}
          </HeroMetric>
        {/snippet}
      </HeroStat>

      {#if downloadFailed}
        <p class="download-error" role="alert">{copy.downloadFailed}</p>
      {/if}

      <div class="view-toolbar">
        <div class="toolbar-actions">
          {#if periodCourseList.length > 0}
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
              onclick={() => loadGrades(true)}
            >
              <RefreshCw size={14} aria-hidden="true" />
            </IconButton>
          </div>
        </div>
      </div>

      {#if viewMode === 'cards'}
        <div class="block-list">
          {#each blocks as block (block.id)}
            {@const label = splitBlockLabel(block.label)}
            {@const courses = blockCourses(block)}
            {@const average = averageOfCourses(courses)}
            {@const open = openBlockIds.includes(block.id)}
            {@const bulletin = blockDocument(block, 'gradeBulletin')}
            {@const transcript = blockDocument(block, 'gradeTranscript')}
            <section class="block-card" class:is-open={open}>
              <h2 class="block-heading">
                <button
                  type="button"
                  class="block-toggle"
                  aria-expanded={open}
                  aria-controls={`block-panel-${block.id}`}
                  onclick={() => toggleBlock(block.id)}
                >
                  <span class="block-chevron" class:is-open={open}>
                    <ChevronDown size={18} aria-hidden="true" />
                  </span>
                  <span class="block-identity">
                    <span class="block-title">{label.title}</span>
                    {#if label.range}<small class="block-range">{label.range}</small>{/if}
                  </span>
                  <span class="block-average" title={copy.blockAverage}>
                    <strong>{formatGrade(average)}</strong>
                    <small>/20</small>
                  </span>
                </button>
              </h2>

              <div id={`block-panel-${block.id}`} class="block-panel" hidden={!open}>
                {#if bulletin || transcript}
                  <div class="block-documents">
                    {#each [bulletin, transcript].filter((document) => document !== null) as document (document.requestPath)}
                      <button
                        type="button"
                        class="document-button"
                        disabled={downloadingPath === document.requestPath}
                        onclick={() => downloadBlockDocument(document)}
                      >
                        {#if downloadingPath === document.requestPath}
                          <Spinner size={14} />
                        {:else if document.kind === 'gradeBulletin'}
                          <FileText size={14} aria-hidden="true" />
                        {:else}
                          <Download size={14} aria-hidden="true" />
                        {/if}
                        {document.kind === 'gradeBulletin' ? copy.bulletin : copy.transcript}
                      </button>
                    {/each}
                  </div>
                {/if}

                {#if courses.length === 0}
                  <p class="block-empty">{copy.blockEmpty}</p>
                {:else}
                  {#each block.sections as section, sectionIndex (section.label ?? sectionIndex)}
                    {#if section.label}
                      <h3 class="section-title">{section.label}</h3>
                    {/if}
                    <div class="course-grid">
                      {#each section.courses as course (course.id)}
                        {@const average = courseAverage(course)}
                        <article class="course-card" style:--subject-color={getSubjectColor(course.name)}>
                          <header class="course-header">
                            <div class="course-identity">
                              <h4 class="course-name" title={course.name}>{course.name}</h4>
                              {#if course.code}<small class="course-code">{course.code}</small>{/if}
                            </div>
                            <span class="course-average" class:is-empty={average === null}>
                              <strong>{formatGrade(average)}</strong>
                              <small>/20</small>
                            </span>
                          </header>

                          {#if course.evaluations.length === 0}
                            <p class="course-empty">{copy.noGradeYet}</p>
                          {:else}
                            <ul class="evaluation-list">
                              {#each course.evaluations as evaluation, index (`${course.id}-${index}`)}
                                <li class="evaluation">
                                  <span class="evaluation-score">
                                    {formatEvaluationScore(evaluation)}
                                  </span>
                                  <span class="evaluation-identity">
                                    <span class="evaluation-label">{evaluation.label}</span>
                                    {#if evaluation.weight}
                                      <small class="evaluation-weight">{evaluation.weight}</small>
                                    {/if}
                                  </span>
                                </li>
                                {#each evaluation.children as child, childIndex (`${course.id}-${index}-${childIndex}`)}
                                  <li class="evaluation is-child">
                                    <span class="evaluation-score">
                                      {formatEvaluationScore(child)}
                                    </span>
                                    <span class="evaluation-identity">
                                      <span class="evaluation-label">{child.label}</span>
                                      {#if child.weight}
                                        <small class="evaluation-weight">{child.weight}</small>
                                      {/if}
                                    </span>
                                  </li>
                                {/each}
                              {/each}
                            </ul>
                          {/if}
                        </article>
                      {/each}
                    </div>
                  {/each}
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
                  <th scope="col">{copy.tableCourse}</th>
                  <th scope="col">{copy.tableEvaluation}</th>
                  <th scope="col" class="text-right">{copy.tableGrade}</th>
                  <th scope="col" class="text-right">{copy.tableWeight}</th>
                </tr>
              </thead>
              <tbody>
                {#each tableRows as row (row.id)}
                  <tr class:is-child={row.isChild}>
                    <td class="table-secondary-cell">
                      <span class="cell-label">{copy.tableBlock}</span>
                      <span>{row.block}</span>
                    </td>
                    <td>
                      <span class="cell-label">{copy.tableCourse}</span>
                      <strong>{row.course}</strong>
                    </td>
                    <td>
                      <span class="cell-label">{copy.tableEvaluation}</span>
                      <span class="table-evaluation-cell" class:is-child={row.isChild}>
                        {row.label}
                      </span>
                    </td>
                    <td class="text-right">
                      <span class="cell-label">{copy.tableGrade}</span>
                      <span class="table-score-badge">{row.score}</span>
                    </td>
                    <td class="text-right table-secondary-cell">
                      <span class="cell-label">{copy.tableWeight}</span>
                      <span>{row.weight}</span>
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

  /* Year → block accordion, mirroring how the portal groups the grades. */
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

  .block-average {
    display: inline-flex;
    flex-shrink: 0;
    align-items: baseline;
    gap: 0.1rem;
  }

  .block-average strong {
    color: var(--primary-deep);
    font-size: var(--text-xl);
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-heavy);
  }

  .block-average small {
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

  .block-empty,
  .course-empty {
    margin: 0;
    color: var(--muted-foreground);
    font-size: var(--text-sm);
  }

  .section-title {
    margin: var(--space-2) 0 0;
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .course-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-3);
  }

  .course-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-3);
    /* The subject tint arrives as a raw hex from `getSubjectColor`, so it is the
       one colour here that is not a token. */
    background: color-mix(in srgb, var(--subject-color, var(--muted)) 45%, var(--card));
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
  }

  .course-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .course-identity {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 0.1rem;
  }

  .course-name {
    display: -webkit-box;
    margin: 0;
    overflow: hidden;
    color: var(--foreground);
    font-size: var(--text-base);
    font-weight: var(--weight-bold);
    line-height: 1.3;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .course-code {
    color: var(--muted-foreground);
    font-size: var(--text-2xs);
    font-variant-numeric: tabular-nums;
  }

  .course-average {
    display: inline-flex;
    flex-shrink: 0;
    align-items: baseline;
    gap: 0.1rem;
    padding: 0.1rem var(--space-2);
    background: var(--card);
    border-radius: var(--radius-pill);
  }

  .course-average strong {
    color: var(--primary-deep);
    font-size: var(--text-md);
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-heavy);
  }

  .course-average.is-empty strong {
    color: var(--muted-foreground);
  }

  .course-average small {
    color: var(--muted-foreground);
    font-size: var(--text-2xs);
    font-weight: var(--weight-semibold);
  }

  .evaluation-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .evaluation {
    display: flex;
    align-items: baseline;
    gap: var(--space-3);
  }

  /* A sub-evaluation is a detail of the line above it, not a mark of its own. */
  .evaluation.is-child {
    padding-left: var(--space-4);
    opacity: 0.75;
  }

  .evaluation-score {
    flex-shrink: 0;
    min-width: 4.5rem;
    color: var(--foreground);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-bold);
  }

  .evaluation.is-child .evaluation-score {
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
  }

  .evaluation-identity {
    display: flex;
    min-width: 0;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--space-1);
  }

  .evaluation-label {
    color: var(--foreground);
    font-size: var(--text-sm);
  }

  .evaluation.is-child .evaluation-label {
    font-size: var(--text-xs);
  }

  .evaluation-weight {
    color: var(--muted-foreground);
    font-size: var(--text-2xs);
    font-variant-numeric: tabular-nums;
  }

  /* Table view. Base layout is the small-screen card fallback; the real table
     only assembles once there is room for five columns. */
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

  .table-evaluation-cell.is-child {
    color: var(--muted-foreground);
    font-size: var(--text-sm);
  }

  .table-score-badge {
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

    .course-grid {
      grid-template-columns: repeat(auto-fill, minmax(min(100%, 20rem), 1fr));
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

    .data-table tr.is-child td:first-child {
      padding-left: var(--space-5);
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
