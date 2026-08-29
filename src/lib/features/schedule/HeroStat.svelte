<script lang="ts">
  import type { Snippet } from 'svelte';
  import Skeleton from '$lib/components/ui/Skeleton.svelte';
  import { cn } from '$lib/utils';

  type Props = {
    /** Accessible name for the panel — normally the view's own heading. */
    ariaLabel: string;
    /** What the headline number measures. */
    label: string;
    /** Already formatted with the caller's locale; this component never formats. */
    value: string;
    /** Trails the value: `/20`, `%`. */
    unit?: string;
    /** Status chip above the label. */
    badge?: Snippet;
    /** Secondary metric row, built from `HeroMetric`. */
    metrics?: Snippet;
    /**
     * Real series behind the headline, oldest first. Given one, the backdrop
     * curve plots it instead of the decorative line.
     */
    trend?: number[];
    /** What `trend` plots — required for the curve to be announced at all. */
    trendLabel?: string;
    /**
     * Whether to render the background curve. Defaults to true.
     */
    showCurve?: boolean;
    /**
     * Renders the same geometry as placeholders, so the panel does not jump
     * between the first fetch and the data landing.
     */
    loading?: boolean;
    /** Number of secondary metric placeholders shown while loading. */
    loadingMetricCount?: number;
    /** Announced while `loading` is true. */
    loadingLabel?: string;
  };

  const {
    ariaLabel,
    label,
    value,
    unit,
    badge,
    metrics,
    trend,
    trendLabel,
    showCurve = true,
    loading = false,
    loadingMetricCount = 3,
    loadingLabel,
  }: Props = $props();

  const VIEWBOX_WIDTH = 900;
  const VIEWBOX_HEIGHT = 260;
  /** Room kept free so the extremes and the head marker never touch the edges. */
  const PADDING = 40;
  const PADDING_X = 14;

  const DECORATIVE_PATH = 'M-40 45 C120 200 215 10 355 88 S585 238 760 65 S940 120 980 -20';

  const series = $derived((trend ?? []).filter((value) => Number.isFinite(value)));

  const points = $derived.by(() => {
    if (series.length === 0) return [];
    const lowest = Math.min(...series);
    const highest = Math.max(...series);
    const span = highest - lowest;
    const plotWidth = VIEWBOX_WIDTH - PADDING_X * 2;
    const step = series.length > 1 ? plotWidth / (series.length - 1) : 0;

    return series.map((value, index) => {
      // A flat series has no span to scale against, so it rides the middle.
      const ratio = span > 0 ? (value - lowest) / span : 0.5;
      return {
        x: series.length > 1 ? PADDING_X + index * step : VIEWBOX_WIDTH / 2,
        y: VIEWBOX_HEIGHT - PADDING - ratio * (VIEWBOX_HEIGHT - PADDING * 2),
      };
    });
  });

  /** Quadratic segments through the midpoints: smooth, and never overshooting. */
  const trendPath = $derived.by(() => {
    if (points.length === 0) return null;
    if (points.length === 1) {
      const [only] = points;
      return `M${PADDING_X} ${only.y} L${VIEWBOX_WIDTH - PADDING_X} ${only.y}`;
    }

    let path = `M${points[0].x} ${points[0].y}`;
    for (let index = 1; index < points.length - 1; index++) {
      const current = points[index];
      const next = points[index + 1];
      path += ` Q${current.x} ${current.y} ${(current.x + next.x) / 2} ${(current.y + next.y) / 2}`;
    }
    const last = points[points.length - 1];
    return `${path} L${last.x} ${last.y}`;
  });

  const lastPoint = $derived(points.at(-1) ?? { x: 355, y: 88 });

  /**
   * One gradient stop per segment midpoint, coloured by that segment's slope:
   * the line reads as rising or falling before the numbers are read at all. The
   * stops sit at midpoints so neighbouring segments blend instead of banding.
   */
  const gradientStops = $derived.by(() => {
    if (points.length < 2) return [];
    const stops: { offset: number; tone: 'up' | 'down' | 'flat' }[] = [];

    for (let index = 0; index < points.length - 1; index++) {
      const current = points[index];
      const next = points[index + 1];
      // SVG y grows downward, so a lower y is a better mark.
      const delta = current.y - next.y;
      const tone = delta > 0.5 ? 'up' : delta < -0.5 ? 'down' : 'flat';
      const midpoint = (current.x + next.x) / 2 / VIEWBOX_WIDTH;
      if (index === 0) stops.push({ offset: 0, tone });
      stops.push({ offset: midpoint, tone });
      if (index === points.length - 2) stops.push({ offset: 1, tone });
    }

    return stops;
  });

  const lastTone = $derived(gradientStops.at(-1)?.tone ?? 'flat');

  const gradientId = $props.id();

  const heroMetrics =
    'mt-3 grid w-full max-w-[38rem] items-stretch justify-center gap-2' +
    ' grid-cols-[repeat(auto-fit,minmax(min(100%,7rem),1fr))]';

  const markerInk = $derived(
    lastTone === 'up' ? 'bg-success-strong' : lastTone === 'down' ? 'bg-danger-strong' : 'bg-primary-deep'
  );
</script>

<!-- One hero for every scalar headline in the app. Elevation is the colour
     field alone — no border and no shadow under the same surface. -->
<section
  class={cn(
    'relative overflow-hidden rounded-xl bg-muted px-4 py-5',
    'transition-[background,border-color,box-shadow] duration-normal ease-out',
    'md:px-5 md:pt-6 md:pb-5'
  )}
  aria-label={loading ? loadingLabel : ariaLabel}
  role={loading ? 'status' : undefined}
  aria-live={loading ? 'polite' : undefined}
  aria-busy={loading ? 'true' : undefined}
>
  {#if showCurve}
    <svg
      class="hero-curve"
      class:is-plot={trendPath !== null}
      viewBox="0 0 {VIEWBOX_WIDTH} {VIEWBOX_HEIGHT}"
      preserveAspectRatio="none"
      role={trendPath && trendLabel ? 'img' : undefined}
      aria-label={trendPath && trendLabel ? trendLabel : undefined}
      aria-hidden={trendPath && trendLabel ? undefined : true}
    >
      {#if gradientStops.length > 0}
        <defs>
          <linearGradient id={`trend-${gradientId}`} x1="0" y1="0" x2="1" y2="0">
            {#each gradientStops as stop, index (index)}
              <stop
                offset={stop.offset}
                class:stop-up={stop.tone === 'up'}
                class:stop-down={stop.tone === 'down'}
                class:stop-flat={stop.tone === 'flat'}
              />
            {/each}
          </linearGradient>
        </defs>
      {/if}

      <!-- Inline, because the stylesheet's own `stroke` would win over a
           presentation attribute. -->
      <path
        d={trendPath ?? DECORATIVE_PATH}
        style:stroke={gradientStops.length > 0 ? `url(#trend-${gradientId})` : undefined}
      />
      {#if !trendPath}
        <circle cx="355" cy="88" r="7" />
      {/if}
    </svg>

    {#if trendPath}
      <!-- The plot is stretched to the panel, so the head of the curve is marked
           in layout space to stay round. -->
      <span
        class={cn(
          'absolute z-raised size-[0.7rem] -translate-x-1/2 -translate-y-1/2 rounded-full',
          'border-[3px] border-card pointer-events-none',
          markerInk
        )}
        style:left="{(lastPoint.x / VIEWBOX_WIDTH) * 100}%"
        style:top="{(lastPoint.y / VIEWBOX_HEIGHT) * 100}%"
        aria-hidden="true"
      ></span>
    {/if}
  {/if}

  <div class="relative z-raised flex flex-col items-center gap-2 text-center">
    {#if loading}
      <Skeleton shape="block" width="6rem" height="1.5rem" />
      <Skeleton shape="block" width="10rem" height="3.2rem" />
      <div class={heroMetrics}>
        {#each Array(loadingMetricCount) as _, index (index)}
          <Skeleton shape="block" height="2.75rem" />
        {/each}
      </div>
    {:else}
      <div class="flex flex-col items-center gap-2">
        {#if badge}{@render badge()}{/if}
        <span class="text-base font-semibold text-primary-deep">{label}</span>
      </div>

      <p
        class="text-[clamp(var(--text-3xl),6vw,var(--text-4xl))] leading-none font-extrabold
               tracking-[-0.02em] tabular-nums text-foreground"
      >{value}{#if unit}<small
          class="ml-[0.15rem] text-lg leading-[1] font-semibold text-primary-deep"
          >{unit}</small
        >{/if}</p>

      {#if metrics}
        <div class={heroMetrics}>{@render metrics()}</div>
      {/if}
    {/if}
  </div>
</section>

<style>
  /* Decorative signal line. `--primary` is a fill token and never a stroke, so
     the curve takes `--primary-deep` and loses weight through opacity instead. */
  .hero-curve {
    position: absolute;
    inset: -3rem -14rem auto -10rem;
    width: calc(100% + 24rem);
    height: 14rem;
    opacity: 0.42;
    pointer-events: none;
  }

  /* A plotted series is data, not decoration: it fills the panel exactly and
     carries more contrast than the idle backdrop. */
  .hero-curve.is-plot {
    inset: 0;
    width: 100%;
    height: 100%;
    opacity: 0.75;
  }

  .hero-curve path {
    fill: none;
    stroke: var(--primary-deep);
    stroke-linecap: round;
    stroke-width: 4;
  }

  .hero-curve.is-plot path {
    stroke-linejoin: round;
    vector-effect: non-scaling-stroke;
    stroke-width: 3;
  }

  .hero-curve circle {
    fill: var(--primary-deep);
    stroke: var(--card);
    stroke-width: 4;
  }

  /* Slope colours. Green rewards a rising run, red flags a falling one; a flat
     run stays on the neutral line colour so only real movement is coloured.
     `stop-color` on a generated <stop> has no utility equivalent. */
  .stop-up {
    stop-color: var(--success);
  }

  .stop-down {
    stop-color: var(--danger);
  }

  .stop-flat {
    stop-color: var(--primary-deep);
  }
</style>
