<script lang="ts">
  /**
   * DEV-ONLY PROTOTYPE — structure B, "Le continuum".
   *
   * The scope stops being a view and becomes a zoom level of one time grid.
   * The consequence that matters: at every zoom the period is laid out to fit
   * the height it was given, so the week no longer scrolls on two axes — it
   * does not scroll at all. Blocks are positioned in percentages of the body,
   * which is why the same drawing works at 61px an hour and at 12px an hour.
   *
   * NOTE for the build: the category fill map below duplicates KindBadge's
   * mapping because a block is a fill and the badge is a chip. If this
   * structure is locked, that mapping moves into KindBadge as a `fill`
   * variant instead of living in a view — DESIGN.md keeps category colour in
   * one component on purpose.
   */
  import { onMount } from 'svelte';
  import { CalendarOff } from 'lucide-svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import CourseDetailModal from '$lib/features/schedule/CourseDetailModal.svelte';
  import DatePickerSheet from '$lib/features/schedule/DatePickerSheet.svelte';
  import {
    blockGeometry,
    layoutDay,
    ratioInWindow,
    timeWindowFor,
    windowHours,
  } from '$lib/features/schedule/calendar-layout';
  import type { PositionedEvent } from '$lib/features/schedule/calendar-layout';
  import { monthGridDays } from '$lib/features/schedule/calendar-navigation.svelte';
  import { uppercaseTiny } from '$lib/features/schedule/calendar-styles';
  import {
    courseCategory,
    eventTitle,
    getEventStatus,
    isCancelled,
    parseRoomAndTeacher,
  } from '$lib/features/schedule/course-utils';
  import { addDays, isSameDay, isSameMonth, startOfWeek } from '$lib/features/schedule/date-utils';
  import type { CalendarEvent } from '$lib/features/schedule/types';
  import { cn } from '$lib/utils';
  import ProtoShell from './ProtoShell.svelte';
  import ProtoTopBar from './ProtoTopBar.svelte';
  import { ProtoModel } from './proto-model.svelte';
  import { swipe } from './swipe';

  const model = new ProtoModel('week');
  const nav = model.navigation;

  onMount(model.startClock.bind(model));

  let detail = $state<CalendarEvent | null>(null);
  let pickerOpen = $state(false);

  const fills = {
    lecture: 'bg-category-lecture-surface text-category-lecture-text',
    tutorial: 'bg-category-tutorial-surface text-category-tutorial-text',
    lab: 'bg-category-lab-surface text-category-lab-text',
    exam: 'bg-category-exam-surface text-category-exam-text',
    project: 'bg-category-project-surface text-category-project-text',
    other: 'bg-category-other-surface text-category-other-text',
  } as const;

  /**
   * A month mark carries no text, so it is drawn as the pale field plus a 1px
   * saturated edge rather than as a saturated block. Filling thirty cells with
   * the ink tone was legible and wrong: it turned a quiet light system into a
   * Gantt chart. The edge gives the precision the fill alone lacked at 4px,
   * and `border-current` picks it up from the pair's own text tone, so there is
   * no second colour table.
   */
  const markEdge = 'border border-current';

  /**
   * A course that is over drops its category colour instead of being faded.
   * `opacity` on a block fades its text too, and an 11px label at 64% on a
   * pale field falls under 4.5:1 — so the past was being drawn by breaking the
   * contrast floor. This is the incumbent view's own answer to the same state,
   * and it earns something: nobody scans the past by category, so spending the
   * palette only on what is still coming makes the future the thing that pops.
   */
  const spentFill = 'bg-surface-sunken text-muted-foreground';
  const spentEdge = 'border border-border-subtle';

  /**
   * Two characters per category, so the month says *what* and not only *when*.
   * It also removes a real defect: before this the category was carried by hue
   * alone at that size, and DESIGN.md forbids a state communicated by colour
   * only. CM / TD / TP are the codes the school already uses.
   */
  const codes = {
    lecture: 'CM',
    tutorial: 'TD',
    lab: 'TP',
    exam: 'EX',
    project: 'PR',
    other: '••',
  } as const;

  /** Width of the month's hour gutter. Narrower than the week's: three labels, not eleven. */
  const MONTH_GUTTER = '2.1rem';

  /** The cell's day-number line, in px. Both the gutter and the cells reserve it. */
  const MONTH_HEADER_PX = 16;
  /** `pt-0.5` + `pb-1` on a month cell. */
  const MONTH_CELL_PADDING_PX = 6;

  const columns = $derived.by(() => {
    if (nav.scope === 'day') return [nav.activeDate];
    return model.weekDays;
  });

  /** Whole weeks of the displayed month, as rows of the month zoom. */
  const monthRows = $derived.by(() => {
    const cells = monthGridDays(nav.anchorDate);
    const rows: Date[][] = [];
    for (let index = 0; index < cells.length; index += 7) {
      rows.push(cells.slice(index, index + 6));
    }
    return rows;
  });

  const gridEvents = $derived.by(() =>
    nav.scope === 'month'
      ? monthRows.flat().flatMap((day) => model.eventsForDay(day))
      : columns.flatMap((day) => model.eventsForDay(day))
  );

  const timeWindow = $derived(timeWindowFor(gridEvents));
  const gridHours = $derived(windowHours(timeWindow));
  const spanHours = $derived((timeWindow.endMinutes - timeWindow.startMinutes) / 60);

  /**
   * Measured, not guessed. Whether a mark can carry text depends on how many
   * pixels an hour is worth in the month, which depends on the window and on
   * the height the bar left the grid — so a threshold in minutes would clip
   * text on one window and waste room on another.
   */
  let monthGridHeight = $state(0);

  const monthBandHeightPx = $derived.by(() => {
    if (monthGridHeight === 0 || monthRows.length === 0) return 0;
    const rowHeight = (monthGridHeight - (monthRows.length - 1)) / monthRows.length;
    return Math.max(0, rowHeight - MONTH_HEADER_PX - MONTH_CELL_PADDING_PX);
  });

  function markHeightPx(positioned: PositionedEvent): number {
    const minutes = positioned.toMinutes - positioned.fromMinutes;
    const windowMinutes = timeWindow.endMinutes - timeWindow.startMinutes;
    if (windowMinutes <= 0) return 0;
    return (minutes / windowMinutes) * monthBandHeightPx;
  }

  /**
   * The floor below which the hours stop being readable and the body starts
   * scrolling instead of squeezing. It is per zoom because the day zoom is
   * where a block has to carry three lines of text.
   */
  const minHourRem = $derived(nav.scope === 'day' ? 3 : nav.scope === 'week' ? 2 : 0.75);

  const offToday = $derived(!isSameDay(nav.activeDate, model.now));

  const hasEvents = $derived(gridEvents.length > 0);

  /**
   * Three anchors instead of the week's eleven: the band's first hour, its
   * middle, its last. Enough to decode a mark's position, few enough that the
   * gutter does not become the loudest thing in the month.
   */
  const monthAnchors = $derived.by(() => {
    const hours = gridHours;
    if (hours.length === 0) return [];
    const picks: { index: number; align: string }[] = [
      { index: 0, align: 'translate-y-0' },
      { index: Math.floor((hours.length - 1) / 2), align: '-translate-y-1/2' },
      { index: hours.length - 1, align: '-translate-y-full' },
    ];
    return picks.map(({ index, align }) => ({
      hour: hours[index],
      // The `h` is not decoration: a cell prints its course count as a bare
      // number in the same size, and `8` beside `2` reads as two counts.
      label: `${hours[index]} h`,
      ratio: index / spanHours,
      align,
    }));
  });

  /** Where the middle anchor sits, ruled across every cell as the shared axis. */
  const middayRatio = $derived(monthAnchors[1]?.ratio ?? 0.5);

  function nowRatio(day: Date): number | null {
    return isSameDay(day, model.now) ? ratioInWindow(model.now, timeWindow) : null;
  }
</script>

<!-- A block at day or week zoom. What it can print is decided by the zoom, not
     by a breakpoint: the same component draws three lines at 61px an hour and
     one at 24px. -->
{#snippet block(positioned: PositionedEvent, dense: boolean)}
  {@const event = positioned.event}
  {@const geometry = blockGeometry(positioned, timeWindow)}
  {@const status = getEventStatus(event, model.now)}
  {@const details = parseRoomAndTeacher(event)}
  {@const cancelled = isCancelled(event)}
  {@const live = status === 'live' && !cancelled}
  <!-- A block that shares its column with another has roughly 25px of width at
       week zoom, and no type size makes a course name readable in that. It
       keeps its hour and its category fill, and says the rest when tapped. -->
  {@const named = !dense || positioned.lanes === 1}

  <button
    type="button"
    class={cn(
      'absolute flex min-h-(--tap-min) min-w-0 flex-col overflow-hidden rounded-xs border',
      'text-start transition-control active:scale-(--press-scale) fine-hover:border-primary-deep',
      dense ? 'gap-0 px-1 py-0.5' : 'gap-[0.15rem] px-2 py-1.5',
      status === 'finished' ? spentFill : fills[courseCategory(event.kind)],
      live ? 'border-primary-deep' : 'border-transparent',
      cancelled && 'border-danger-strong border-dashed'
    )}
    style:top={`${geometry.top}%`}
    style:height={`${geometry.height}%`}
    style:left={`calc(${geometry.left}% + ${geometry.left > 0 ? '2px' : '0px'})`}
    style:width={`calc(${geometry.width}% - 2px)`}
    aria-label={named
      ? undefined
      : `${eventTitle(event)}, ${model.format.eventTimeRange(event)}${details.room ? `, ${details.room}` : ''}`}
    onclick={() => (detail = event)}
  >
    <!-- The live badge rides beside the hour rather than at the block's
         bottom. Pinned there it sat exactly where the now-line crosses a
         running course, and a status chip with a rule struck through it reads
         as a rendering bug. -->
    <span class="flex min-w-0 items-center gap-1.5">
      <span class="shrink-0 text-2xs leading-[1.15] font-bold tabular-nums">
        <!-- A shared column is ~25px wide: `08:00` truncates to `08:0`, which
             reads as a bug. The compact hour fits and stays true. -->
        {named
          ? model.format.timeFormatter.format(new Date(event.startsAt))
          : `${new Date(event.startsAt).getHours()}h`}
      </span>
      {#if !dense && live}
        <Badge tone="live" dot>En cours</Badge>
      {/if}
    </span>
    {#if named}
      <span
        class={cn(
          'min-w-0 font-extrabold',
          dense ? 'text-2xs leading-[1.15] hyphens-auto' : 'text-xs leading-[1.25] wrap-anywhere',
          cancelled && 'line-through'
        )}
        lang="fr">{eventTitle(event)}</span
      >
    {/if}
    {#if !dense && details.room}
      <span class="min-w-0 truncate text-2xs leading-[1.2] font-medium">{details.room}</span>
    {/if}
  </button>
{/snippet}

<!-- One day column: the blocks and, on today, the line that says where now is.
     The column owns its grid placement so the absolutely positioned blocks
     resolve their percentages against a box that actually has a height. -->
{#snippet column(day: Date, dense: boolean, index: number)}
  {@const ratio = nowRatio(day)}
  <div class="relative min-w-0" style:grid-column={index + 2} style:grid-row="1">
    {#each layoutDay(model.eventsForDay(day)) as positioned (positioned.event.id)}
      {@render block(positioned, dense)}
    {/each}
    {#if ratio !== null}
      <!-- DESIGN.md wants the now-line labelled with its hour. At week zoom a
           column is 53px and the pill would cover the course under it, so the
           label rides only where there is room for it. -->
      <div
        class="pointer-events-none absolute inset-x-0 z-raised border-t-2 border-primary-deep"
        style:top={`${ratio * 100}%`}
      >
        <span class="absolute -start-1 -top-[3px] size-1.5 rounded-full bg-primary-deep"></span>
        {#if !dense}
          <span
            class="absolute end-0 -translate-y-1/2 rounded-pill bg-primary-deep px-1.5
                   text-2xs font-bold tabular-nums text-card"
            >{model.format.timeFormatter.format(model.now)}</span
          >
        {/if}
        <span class="sr-only">Heure actuelle</span>
      </div>
    {/if}
  </div>
{/snippet}

<ProtoShell
  scope={nav.scope}
  {offToday}
  onScope={nav.setScope}
  onMove={nav.movePeriod}
  onToday={nav.goToToday}
>
  <ProtoTopBar
    label={model.format.periodLabel}
    fetchedAt={model.fetchedAt}
    onPick={() => (pickerOpen = true)}
  />

  <!-- No zoom scrolls sideways, so the horizontal axis belongs to the period
       at every one of them — including the week, where the current view has
       to disable the gesture. -->
  <div
    class="relative flex min-h-0 flex-1 flex-col px-2 pt-2 pb-1"
    use:swipe={{ onSwipe: nav.movePeriod }}
  >
    <!-- Empty does not remove the grid. This structure's whole claim is that
         the time axis never leaves, and an empty band that still reads
         "Monday, 08:00 to 18:00, nothing in it" says more than a card standing
         where the grid was. The statement rides over it, and does not take the
         pointer so the period can still be swiped away. -->
    {#if !hasEvents}
      <div
        class="pointer-events-none absolute inset-0 z-raised flex items-center justify-center p-6"
        role="status"
      >
        <div
          class="flex max-w-[16rem] flex-col items-center gap-2 rounded-lg border
                 border-border-subtle bg-card/92 px-4 py-4 text-center shadow-sm
                 backdrop-blur-[6px]"
        >
          <CalendarOff size={22} class="text-primary-deep" aria-hidden="true" />
          <p class="text-sm leading-tight font-extrabold text-foreground">
            Aucun cours sur cette période
          </p>
          <p class="text-2xs leading-relaxed text-muted-foreground">
            Le portail n’a renvoyé aucun créneau. Balaye pour changer de période.
          </p>
        </div>
      </div>
    {/if}

    {#if nav.scope === 'month'}
      <!-- Month zoom: the same drawing, five times smaller — which only works
           if the vertical axis is still readable. The first pass drew bars with
           no hour reference at all, so a bar's height on the cell meant
           nothing: pretty, undecodable. Three things fix it.
           1. Every week row carries the hour scale in a gutter, the week
              zoom's gutter with three anchors instead of eleven, and the
              middle anchor is ruled across every cell so morning and
              afternoon are two readable halves.
           2. The month is one field split by hairlines, not thirty bordered
              cards. `gap-px` over the border colour draws every rule once,
              which is also why no cell needs a border of its own.
           3. The marks keep the pale category field and gain a 1px saturated
              edge. Filled with the ink tone they were legible and wrong — a
              light system turned into a Gantt chart — and the pale fill alone
              dissolved at 4px. -->
      <div class="flex min-h-0 flex-1 flex-col gap-1.5">
        <div
          class="grid shrink-0 gap-px"
          style:grid-template-columns={`${MONTH_GUTTER} repeat(6, minmax(0, 1fr))`}
        >
          <span></span>
          {#each model.weekDays as day (day.toISOString())}
            <!-- Two letters, not one: in French lundi, mardi and mercredi all
                 start with the same letter and a one-letter header is a
                 guessing game. -->
            <span class={cn(uppercaseTiny, 'text-center text-muted-foreground')}
              >{model.format.weekdayShortFormatter.format(day).slice(0, 2)}</span
            >
          {/each}
        </div>

        <div
          class="min-h-0 flex-1 overflow-hidden rounded-lg border border-border-subtle
                 bg-border-subtle"
        >
          <div
            class="grid h-full gap-px"
            style:grid-template-columns={`${MONTH_GUTTER} repeat(6, minmax(0, 1fr))`}
            style:grid-template-rows={`repeat(${monthRows.length}, minmax(0, 1fr))`}
            bind:clientHeight={monthGridHeight}
          >
            {#each monthRows as row, rowIndex (rowIndex)}
              <!-- The gutter reserves the same header height as a cell, so its
                   anchors land on the band and not beside it. -->
              <div class="flex min-h-0 flex-col bg-card pe-1">
                <span class="block h-4 shrink-0"></span>
                <span class="relative min-h-0 flex-1">
                  {#each monthAnchors as anchor (anchor.hour)}
                    <span
                      class={cn(
                        'absolute end-0 text-2xs leading-none font-semibold tabular-nums',
                        'text-muted-foreground',
                        anchor.align
                      )}
                      style:top={`${anchor.ratio * 100}%`}>{anchor.label}</span
                    >
                  {/each}
                </span>
              </div>

              {#each row as day (day.toISOString())}
                {@const outside = !isSameMonth(day, nav.anchorDate)}
                {@const dayEvents = outside ? [] : model.eventsForDay(day)}
                {@const isToday = isSameDay(day, model.now)}
                {@const isActive = isSameDay(day, nav.activeDate)}
                <button
                  type="button"
                  class={cn(
                    'relative flex min-h-0 flex-col px-1 pt-0.5 pb-1 text-start',
                    'transition-control active:scale-(--press-scale)',
                    'fine-hover:bg-muted',
                    outside ? 'bg-background' : 'bg-card',
                    isToday && 'bg-muted',
                    isActive && 'outline-2 -outline-offset-2 outline-primary-deep'
                  )}
                  aria-label={`${model.format.dayFormatter.format(day)}, ${dayEvents.length} cours`}
                  aria-pressed={isActive}
                  onclick={() => {
                    nav.selectDate(day);
                    nav.setScope('week');
                  }}
                >
                  <span class="flex h-4 shrink-0 items-center justify-between gap-1">
                    <span
                      class={cn(
                        'text-xs leading-none tabular-nums',
                        outside
                          ? 'font-semibold text-muted-foreground/60'
                          : isToday
                            ? 'font-extrabold text-primary-deep'
                            : 'font-bold text-foreground'
                      )}>{day.getDate()}</span
                    >
                    {#if dayEvents.length > 0}
                      <span
                        class="text-2xs leading-none font-semibold tabular-nums text-muted-foreground"
                        >{dayEvents.length}</span
                      >
                    {/if}
                  </span>

                  <span class="relative min-h-0 w-full flex-1">
                    {#if !outside}
                      <span
                        class="absolute inset-x-0 border-t border-border-subtle"
                        style:top={`${middayRatio * 100}%`}
                        aria-hidden="true"
                      ></span>
                    {/if}
                    {#each layoutDay(dayEvents) as positioned (positioned.event.id)}
                      {@const geometry = blockGeometry(positioned, timeWindow)}
                      {@const category = courseCategory(positioned.event.kind)}
                      {@const height = markHeightPx(positioned)}
                      <!-- A mark shares its column when the day overlaps, which
                           halves its width; below that there is no room for a
                           code and the fill goes back to carrying it alone. -->
                      {@const roomy = positioned.lanes === 1}
                      {@const spent = getEventStatus(positioned.event, model.now) === 'finished'}
                      {@const exam = category === 'exam' && !spent}
                      <span
                        class={cn(
                          'absolute flex min-h-[4px] flex-col items-start overflow-hidden',
                          'rounded-[2px] px-[2px]',
                          // Exams are the one category a student scans a month
                          // for, so a coming exam is the single saturated mark
                          // and the rest stays a pale field. A focal point, not
                          // a second colour scheme.
                          exam
                            ? 'border border-category-exam-text bg-category-exam-text text-card'
                            : spent
                              ? cn(spentFill, spentEdge)
                              : cn(fills[category], markEdge),
                          isCancelled(positioned.event) && 'border-dashed opacity-50'
                        )}
                        style:top={`${geometry.top}%`}
                        style:height={`${geometry.height}%`}
                        style:left={`${geometry.left}%`}
                        style:width={`calc(${geometry.width}% - 1px)`}
                      >
                        {#if roomy && height >= 15}
                          <span
                            class={cn(
                              'text-2xs leading-[1.05] font-extrabold',
                              exam && 'tracking-[0.02em]'
                            )}>{codes[category]}</span
                          >
                        {/if}
                        {#if roomy && height >= 30}
                          <span class="text-2xs leading-[1.05] font-normal tabular-nums"
                            >{model.format.timeFormatter.format(
                              new Date(positioned.event.startsAt)
                            )}</span
                          >
                        {/if}
                      </span>
                    {/each}
                  </span>
                </button>
              {/each}
            {/each}
          </div>
        </div>
      </div>
    {:else}
      {#key nav.scope}
        <div class="flex min-h-0 flex-1 flex-col animate-fade-in-fast-forwards">
          {#if nav.scope === 'week'}
            <div
              class="grid shrink-0 gap-1 pb-1"
              style:grid-template-columns={`2.5rem repeat(${columns.length}, minmax(0, 1fr))`}
            >
              <span></span>
              {#each columns as day (day.toISOString())}
                {@const isToday = isSameDay(day, model.now)}
                <button
                  type="button"
                  class={cn(
                    'flex min-h-9 flex-col items-center justify-center rounded-sm border',
                    'transition-control active:scale-(--press-scale)',
                    isToday
                      ? 'border-primary-deep bg-muted text-primary-deep'
                      : 'border-transparent bg-surface-sunken text-muted-foreground'
                  )}
                  onclick={() => {
                    nav.selectDate(day);
                    nav.setScope('day');
                  }}
                >
                  <span class="text-2xs leading-none font-bold uppercase"
                    >{model.format.weekdayShortFormatter.format(day).slice(0, 2)}</span
                  >
                  <span class="text-xs leading-tight font-extrabold tabular-nums"
                    >{day.getDate()}</span
                  >
                </button>
              {/each}
            </div>
          {/if}

          <div
            class="min-h-0 flex-1 overflow-y-auto overscroll-contain"
            style:--min-body={`${minHourRem * spanHours}rem`}
          >
            <div
              class="grid h-full min-h-(--min-body) gap-1"
              style:grid-template-columns={`2.5rem repeat(${columns.length}, minmax(0, 1fr))`}
            >
              <!-- The hour scale and the rules, drawn once for every column. -->
              <div class="relative">
                <!-- The first and last labels are pulled inside the band. A
                     half-line hanging off the bottom is 8px of scroll, and
                     this structure's whole claim is that there is none. -->
                {#each gridHours as hour, index (hour)}
                  <span
                    class={cn(
                      'absolute end-1.5 text-2xs font-semibold tabular-nums text-muted-foreground',
                      index === 0
                        ? 'translate-y-0'
                        : index === gridHours.length - 1
                          ? '-translate-y-full'
                          : '-translate-y-1/2'
                    )}
                    style:top={`${(index / spanHours) * 100}%`}
                  >
                    {model.format.timeFormatter.format(new Date(2024, 0, 1, hour, 0))}
                  </span>
                {/each}
              </div>

              <div
                class="pointer-events-none relative col-start-2 col-end-[-1] row-start-1"
                aria-hidden="true"
              >
                {#each gridHours as hour, index (hour)}
                  <span
                    class="absolute inset-x-0 border-t border-border-subtle"
                    style:top={`${(index / spanHours) * 100}%`}
                  ></span>
                {/each}
              </div>

              {#each columns as day, index (day.toISOString())}
                {@render column(day, nav.scope === 'week', index)}
              {/each}
            </div>
          </div>
        </div>
      {/key}
    {/if}
  </div>

</ProtoShell>

{#if pickerOpen}
  <DatePickerSheet
    activeDate={nav.activeDate}
    now={model.now}
    format={model.format}
    eventsForDay={model.eventsForDay}
    onPick={(date) => {
      nav.pickDate(date);
      pickerOpen = false;
    }}
    onClose={() => (pickerOpen = false)}
  />
{/if}

<CourseDetailModal
  event={detail}
  locale="fr"
  now={model.now}
  onClose={() => (detail = null)}
/>
