<script lang="ts">
  import type { Snippet } from 'svelte';
  import Skeleton from '$lib/components/ui/Skeleton.svelte';

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
    /**
     * Optional presence rate (0 to 100). When set, applies an adaptive background
     * ranging from red (0%) to golden yellow (50%) to green (100%).
     */
    presenceRate?: number;
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
    presenceRate,
  }: Props = $props();

  import { presenceColorStyle } from './absence-utils';

  const isPresenceActive = $derived(
    presenceRate !== undefined && Number.isFinite(presenceRate) && !loading
  );

  const presenceStyle = $derived(
    isPresenceActive ? presenceColorStyle(presenceRate!) : undefined
  );

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
</script>

<section
  class="hero-stat"
  class:has-presence={isPresenceActive}
  style={presenceStyle}
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
        class="hero-marker"
        class:is-up={lastTone === 'up'}
        class:is-down={lastTone === 'down'}
        style:left="{(lastPoint.x / VIEWBOX_WIDTH) * 100}%"
        style:top="{(lastPoint.y / VIEWBOX_HEIGHT) * 100}%"
        aria-hidden="true"
      ></span>
    {/if}
  {/if}

  <div class="hero-inner">
    {#if loading}
      <Skeleton shape="block" width="6rem" height="1.5rem" />
      <Skeleton shape="block" width="10rem" height="3.2rem" />
      <div class="hero-metrics">
        {#each Array(loadingMetricCount) as _, index (index)}
          <Skeleton shape="block" height="2.75rem" />
        {/each}
      </div>
    {:else}
      <div class="hero-top">
        {#if badge}{@render badge()}{/if}
        <span class="hero-label">{label}</span>
      </div>

      <p class="hero-value">{value}{#if unit}<small>{unit}</small>{/if}</p>

      {#if metrics}
        <div class="hero-metrics">{@render metrics()}</div>
      {/if}
    {/if}
  </div>
</section>

<style>
  /* One hero for every scalar headline in the app. Elevation is the colour
     field alone — no border and no shadow under the same surface. */
  .hero-stat {
    position: relative;
    overflow: hidden;
    padding: var(--space-5) var(--space-4);
    background: var(--muted);
    border-radius: var(--radius-xl);
    transition:
      background var(--duration-normal) var(--ease-out),
      border-color var(--duration-normal) var(--ease-out),
      box-shadow var(--duration-normal) var(--ease-out);
  }

  /* Adaptive Presence Card: red at 0%, amber/yellow at 50%, green at 100% */
  .hero-stat.has-presence {
    background:
      radial-gradient(
        ellipse 90% 75% at 50% -15%,
        hsl(var(--presence-hue) 85% 55% / 0.24) 0%,
        transparent 70%
      ),
      radial-gradient(
        ellipse 70% 50% at 100% 100%,
        hsl(var(--presence-hue) 80% 50% / 0.12) 0%,
        transparent 65%
      ),
      linear-gradient(
        155deg,
        hsl(var(--presence-hue) 55% 96% / 0.95) 0%,
        hsl(var(--presence-hue) 45% 91% / 0.9) 100%
      );
    border: 1px solid hsl(var(--presence-hue) 60% 45% / 0.28);
    box-shadow: 0 4px 20px -2px hsl(var(--presence-hue) 70% 40% / 0.12);
  }

  .hero-stat.has-presence .hero-curve path {
    stroke: hsl(var(--presence-hue) 65% 36%);
  }

  .hero-stat.has-presence .hero-curve circle {
    fill: hsl(var(--presence-hue) 65% 36%);
    stroke: var(--card);
  }

  .hero-stat.has-presence .hero-label {
    color: hsl(var(--presence-hue) 70% 28%);
  }

  .hero-stat.has-presence .hero-value small {
    color: hsl(var(--presence-hue) 70% 32%);
  }

  .hero-stat.has-presence :global(.hero-metric) {
    background: color-mix(in oklch, var(--card) 82%, transparent);
    backdrop-filter: blur(8px);
    border: 1px solid hsl(var(--presence-hue) 50% 50% / 0.18);
  }

  :global([data-theme="dark"]) .hero-stat.has-presence,
  :global(.dark) .hero-stat.has-presence {
    background:
      radial-gradient(
        ellipse 90% 75% at 50% -15%,
        hsl(var(--presence-hue) 85% 50% / 0.28) 0%,
        transparent 70%
      ),
      radial-gradient(
        ellipse 70% 50% at 100% 100%,
        hsl(var(--presence-hue) 75% 45% / 0.18) 0%,
        transparent 65%
      ),
      linear-gradient(
        155deg,
        hsl(var(--presence-hue) 45% 15% / 0.95) 0%,
        hsl(var(--presence-hue) 40% 10% / 0.9) 100%
      );
    border: 1px solid hsl(var(--presence-hue) 60% 50% / 0.32);
    box-shadow: 0 4px 24px -2px hsl(var(--presence-hue) 70% 30% / 0.25);
  }

  :global([data-theme="dark"]) .hero-stat.has-presence .hero-curve path,
  :global(.dark) .hero-stat.has-presence .hero-curve path {
    stroke: hsl(var(--presence-hue) 75% 65%);
  }

  :global([data-theme="dark"]) .hero-stat.has-presence .hero-curve circle,
  :global(.dark) .hero-stat.has-presence .hero-curve circle {
    fill: hsl(var(--presence-hue) 75% 65%);
    stroke: hsl(var(--presence-hue) 40% 12%);
  }

  :global([data-theme="dark"]) .hero-stat.has-presence .hero-label,
  :global(.dark) .hero-stat.has-presence .hero-label {
    color: hsl(var(--presence-hue) 80% 75%);
  }

  :global([data-theme="dark"]) .hero-stat.has-presence .hero-value small,
  :global(.dark) .hero-stat.has-presence .hero-value small {
    color: hsl(var(--presence-hue) 80% 70%);
  }

  :global([data-theme="dark"]) .hero-stat.has-presence :global(.hero-metric),
  :global(.dark) .hero-stat.has-presence :global(.hero-metric) {
    background: color-mix(in oklch, var(--card) 60%, transparent);
    border-color: hsl(var(--presence-hue) 60% 60% / 0.22);
  }

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

  /* Slope colours. Green rewards a rising run, red flags a falling one; a flat
     run stays on the neutral line colour so only real movement is coloured. */
  .stop-up {
    stop-color: var(--success);
  }

  .stop-down {
    stop-color: var(--danger);
  }

  .stop-flat {
    stop-color: var(--primary-deep);
  }

  .hero-marker {
    position: absolute;
    z-index: var(--z-raised);
    width: 0.7rem;
    height: 0.7rem;
    background: var(--primary-deep);
    border: 3px solid var(--card);
    border-radius: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
  }

  .hero-marker.is-up {
    background: var(--success-strong);
  }

  .hero-marker.is-down {
    background: var(--danger-strong);
  }

  .hero-curve circle {
    fill: var(--primary-deep);
    stroke: var(--card);
    stroke-width: 4;
  }

  .hero-inner {
    position: relative;
    z-index: var(--z-raised);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    text-align: center;
  }

  .hero-top {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
  }

  .hero-label {
    color: var(--primary-deep);
    font-size: var(--text-base);
    font-weight: var(--weight-semibold);
  }

  .hero-value {
    margin: 0;
    color: var(--foreground);
    font-size: clamp(var(--text-3xl), 6vw, var(--text-4xl));
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-heavy);
    line-height: 1;
    letter-spacing: -0.02em;
  }

  .hero-value small {
    margin-left: 0.15rem;
    color: var(--primary-deep);
    font-size: var(--text-lg);
    font-weight: var(--weight-semibold);
  }

  .hero-metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 7rem), 1fr));
    align-items: stretch;
    justify-content: center;
    gap: var(--space-2);
    width: 100%;
    max-width: 38rem;
    margin-top: var(--space-3);
  }

  @media (min-width: 48rem) {
    .hero-stat {
      padding: var(--space-6) var(--space-5) var(--space-5);
    }
  }
</style>
