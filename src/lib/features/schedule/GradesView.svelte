<script lang="ts">
  import { onMount } from 'svelte';
  import {
    BookOpenCheck,
    Download,
    FileText,
    Sparkles,
    TrendingDown,
    TrendingUp,
  } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import DataTable, { cellLabel } from '$lib/components/ui/DataTable.svelte';
  import PageShell from '$lib/components/ui/PageShell.svelte';
  import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
  import HeroMetric from './HeroMetric.svelte';
  import HeroStat from './HeroStat.svelte';
  import AcademicViewSkeleton from './AcademicViewSkeleton.svelte';
  import BlockAccordion from './BlockAccordion.svelte';
  import DocumentPill from './DocumentPill.svelte';
  import PortalResourceView from './PortalResourceView.svelte';
  import YearStepper from './YearStepper.svelte';
  import { cn } from '$lib/utils';
  import {
    createBlockDisclosure,
    createDocumentDownload,
    latestPeriod,
  } from './academic-view.svelte';
  import { createPortalResource } from './portal-resource.svelte';
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
  import { getSubjectColor, splitBlockLabel } from './portal-utils';
  import type {
    GradeBlock,
    GradeEvaluation,
    PortalDocument,
    PortalDocumentKind,
  } from './types';

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
      heading: m.grades_heading(),
      average: m.grade_average(),
      count: m.grade_count(),
      highest: m.grade_highest(),
      lowest: m.grade_lowest(),
      cardsView: m.cards_view(),
      tableView: m.table_view(),
      loading: m.resource_loading(),
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

  const grades = createPortalResource({
    resource: 'grades',
    fallbackErrorCode: 'grades_unavailable',
    heading: () => copy.heading,
    locale: () => locale,
  });

  const disclosure = createBlockDisclosure();
  const downloads = createDocumentDownload();

  $effect(() => {
    refresh = () => grades.load(true);
  });

  onMount(() => {
    void grades.load();
  });

  const periods = $derived(grades.page?.gradePeriods ?? []);

  const selectedPeriod = $derived(
    periods.find((period) => period.id === selectedPeriodId) ?? latestPeriod(periods)
  );

  const blocks = $derived(selectedPeriod?.blocks ?? []);

  const periodCourseList = $derived(selectedPeriod ? periodCourses(selectedPeriod) : []);

  const overallAverage = $derived(averageOfCourses(periodCourseList));
  const highestGrade = $derived(extremeScaledValue(periodCourseList, 'highest'));
  const lowestGrade = $derived(extremeScaledValue(periodCourseList, 'lowest'));
  const evaluationCount = $derived(gradedEvaluations(periodCourseList).length);
  const trendSeries = $derived(cumulativeAverages(periodCourseList));

  const openBlockIds = $derived(disclosure.openIds(selectedPeriod?.id));

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
    return getGradeStatusBadge(overallAverage);
  });

  const viewOptions = $derived([
    { value: 'cards', label: copy.cardsView },
    { value: 'table', label: copy.tableView },
  ]);

  /** One row per evaluation, sub-evaluations included and marked as such. */
  const tableRows = $derived.by(() => {
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

  const uppercaseLabel =
    'text-xs font-bold tracking-[0.04em] uppercase text-muted-foreground';
</script>

<PageShell>
  <PortalResourceView
    resource={grades}
    {locale}
    {onLogout}
    empty={periods.length === 0}
    emptyIcon={BookOpenCheck}
  >
    {#snippet skeleton()}
      <AcademicViewSkeleton ariaLabel={copy.loading} heroLabel={copy.average} />
    {/snippet}

    {#snippet controls()}
      {#if periodCourseList.length > 0}
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

      {#if downloads.failed}
        <p
          class="rounded-md bg-danger-surface px-3 py-2 text-sm font-semibold text-danger-strong"
          role="alert"
        >{copy.downloadFailed}</p>
      {/if}

      {@render toolbar()}

      {#if viewMode === 'cards'}
        <!-- Year to block accordion, mirroring how the portal groups grades. -->
        <div class="flex flex-col gap-3">
          {#each blocks as block (block.id)}
            {@const label = splitBlockLabel(block.label)}
            {@const courses = blockCourses(block)}
            {@const average = averageOfCourses(courses)}
            {@const bulletin = blockDocument(block, 'gradeBulletin')}
            {@const transcript = blockDocument(block, 'gradeTranscript')}
            <BlockAccordion
              panelId={`block-panel-${block.id}`}
              title={label.title}
              range={label.range}
              value={formatGrade(average)}
              unit="/20"
              valueTitle={copy.blockAverage}
              open={openBlockIds.includes(block.id)}
              onToggle={() => disclosure.toggle(selectedPeriod?.id, block.id)}
            >
              {#if bulletin || transcript}
                <div class="flex flex-wrap gap-2">
                  {#each [bulletin, transcript].filter((document) => document !== null) as document (document.requestPath)}
                    <DocumentPill
                      label={document.kind === 'gradeBulletin' ? copy.bulletin : copy.transcript}
                      icon={document.kind === 'gradeBulletin' ? FileText : Download}
                      busy={downloads.requestPath === document.requestPath}
                      onclick={() => downloads.download(document)}
                    />
                  {/each}
                </div>
              {/if}

              {#if courses.length === 0}
                <p class="text-sm text-muted-foreground">{copy.blockEmpty}</p>
              {:else}
                {#each block.sections as section, sectionIndex (section.label ?? sectionIndex)}
                  {#if section.label}
                    <h3 class={cn('mt-2', uppercaseLabel)}>{section.label}</h3>
                  {/if}
                  <div
                    class="grid grid-cols-1 gap-3
                           md:grid-cols-[repeat(auto-fill,minmax(min(100%,20rem),1fr))]"
                  >
                    {#each section.courses as course (course.id)}
                      {@const average = courseAverage(course)}
                      <article
                        class="flex flex-col gap-3 rounded-lg border border-border-subtle
                               bg-subject-veil p-3"
                        style:--subject-color={getSubjectColor(course.name)}
                      >
                        <header class="flex items-start justify-between gap-2">
                          <div class="flex min-w-0 flex-col gap-[0.1rem]">
                            <h4
                              class="line-clamp-2 text-base leading-[1.3] font-bold text-foreground"
                              title={course.name}>{course.name}</h4
                            >
                            {#if course.code}
                              <small class="text-2xs tabular-nums text-muted-foreground"
                                >{course.code}</small
                              >
                            {/if}
                          </div>
                          <span
                            class={cn(
                              'inline-flex shrink-0 items-baseline gap-[0.1rem] rounded-pill',
                              'bg-card px-2 py-[0.1rem]',
                              average === null && '[&>strong]:text-muted-foreground'
                            )}
                          >
                            <strong class="text-md font-extrabold tabular-nums text-primary-deep"
                              >{formatGrade(average)}</strong
                            >
                            <small class="text-2xs font-semibold text-muted-foreground">/20</small>
                          </span>
                        </header>

                        {#if course.evaluations.length === 0}
                          <p class="text-sm text-muted-foreground">{copy.noGradeYet}</p>
                        {:else}
                          <ul class="flex list-none flex-col gap-2">
                            {#each course.evaluations as evaluation, index (`${course.id}-${index}`)}
                              <li class="flex items-baseline gap-3">
                                <span
                                  class="min-w-18 shrink-0 text-sm font-bold tabular-nums
                                         text-foreground"
                                >
                                  {formatEvaluationScore(evaluation)}
                                </span>
                                <span
                                  class="flex min-w-0 flex-wrap items-baseline gap-1"
                                >
                                  <span class="text-sm text-foreground">{evaluation.label}</span>
                                  {#if evaluation.weight}
                                    <small class="text-2xs tabular-nums text-muted-foreground">{evaluation.weight}</small>
                                  {/if}
                                </span>
                              </li>
                              {#each evaluation.children as child, childIndex (`${course.id}-${index}-${childIndex}`)}
                                <!-- A sub-evaluation is a detail of the line above
                                     it, not a mark of its own. -->
                                <li class="flex items-baseline gap-3 pl-4 opacity-75">
                                  <span
                                    class="min-w-18 shrink-0 text-xs font-semibold tabular-nums
                                           text-foreground"
                                  >
                                    {formatEvaluationScore(child)}
                                  </span>
                                  <span
                                    class="flex min-w-0 flex-wrap items-baseline gap-1"
                                  >
                                    <span class="text-xs text-foreground">{child.label}</span>
                                    {#if child.weight}
                                      <small class="text-2xs tabular-nums text-muted-foreground">{child.weight}</small>
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
            </BlockAccordion>
          {/each}
        </div>
      {:else}
        <DataTable>
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
                <td class="text-sm text-muted-foreground">
                  <span class={cellLabel}>{copy.tableBlock}</span>
                  <span>{row.block}</span>
                </td>
                <td>
                  <span class={cellLabel}>{copy.tableCourse}</span>
                  <strong>{row.course}</strong>
                </td>
                <td>
                  <span class={cellLabel}>{copy.tableEvaluation}</span>
                  <span class={row.isChild ? 'text-sm text-muted-foreground' : ''}>
                    {row.label}
                  </span>
                </td>
                <td class="text-right">
                  <span class={cellLabel}>{copy.tableGrade}</span>
                  <span class="inline-block rounded-sm bg-muted px-2 py-[0.2rem] font-bold tabular-nums text-primary-deep">{row.score}</span>
                </td>
                <td class="text-right text-sm text-muted-foreground">
                  <span class={cellLabel}>{copy.tableWeight}</span>
                  <span>{row.weight}</span>
                </td>
              </tr>
            {/each}
          </tbody>
        </DataTable>
      {/if}
    {/snippet}
  </PortalResourceView>
</PageShell>
