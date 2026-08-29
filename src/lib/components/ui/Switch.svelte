<script lang="ts">
  import { cn } from '$lib/utils';

  type Props = {
    checked: boolean;
    /**
     * Names the control for screen readers. Omit it only when a visible
     * `<label for>` already points at `id` — a switch with neither is unnamed.
     */
    label?: string;
    /** Lets a visible `<label for>` bind to the control; `button` is labelable. */
    id?: string;
    disabled?: boolean;
    /**
     * A write is in flight. The caller keeps showing the state the store last
     * confirmed rather than the one that was asked for, so a failed write never
     * leaves the reader believing a setting changed.
     */
    busy?: boolean;
    onChange: (checked: boolean) => void;
    class?: string;
  };

  const {
    checked,
    label,
    id,
    disabled = false,
    busy = false,
    onChange,
    class: className
  }: Props = $props();

  // The track is 46px of inner width holding a 20px thumb inside 3px of inline
  // padding, so the travel is exactly 20px. Stated here because the two values
  // below have to move together.
  const tracks = {
    on: 'border-primary-deep bg-primary-deep',
    off: 'border-muted-strong bg-muted'
  } as const satisfies Record<'on' | 'off', string>;
</script>

<button
  {id}
  type="button"
  role="switch"
  class={cn(
    // The button is the 44px hit target and carries the focus outline; the
    // track inside it is the smaller visual box, because a settings row does
    // not want a control as tall as a toolbar button.
    'ui-switch inline-flex min-h-(--tap-min) flex-none items-center justify-center',
    'rounded-pill border border-transparent bg-transparent px-1',
    'transition-control disabled:opacity-62 enabled:active:scale-(--press-scale)',
    className
  )}
  aria-checked={checked}
  aria-label={label}
  aria-busy={busy ? 'true' : undefined}
  disabled={disabled || busy}
  onclick={() => onChange(!checked)}
>
  <span
    class={cn(
      'flex h-7 w-12 flex-none items-center rounded-pill border px-[0.1875rem]',
      'transition-control',
      checked ? tracks.on : tracks.off
    )}
    aria-hidden="true"
  >
    <span
      class={cn(
        'size-5 flex-none rounded-pill bg-card',
        // `transition-control` covers colour and scale only, so the travel of
        // the thumb declares its own transition.
        'transition-transform duration-fast ease-out',
        checked ? 'translate-x-5' : 'translate-x-0'
      )}
    ></span>
  </span>
</button>
