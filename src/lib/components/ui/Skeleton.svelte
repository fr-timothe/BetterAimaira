<script lang="ts">
  import { cn } from '$lib/utils';

  type Props = {
    /** `text` and `title` size themselves; `block` takes the height you give it. */
    shape?: 'text' | 'title' | 'block' | 'circle';
    /** CSS length. Defaults to filling the inline axis. */
    width?: string;
    /** CSS length. Only read for `block` and `circle`. */
    height?: string;
    class?: string;
  };

  const { shape = 'text', width, height, class: className }: Props = $props();

  // One skeleton grammar for the whole app: an opacity pulse on a tinted field.
  // Placeholders never carry a shimmer AND a pulse.
  const shapes = {
    text: 'h-3 w-full',
    title: 'h-[1.15rem] w-3/5 rounded-xs',
    block: 'h-20 w-full rounded-lg',
    circle: 'size-11 rounded-full'
  } as const satisfies Record<NonNullable<Props['shape']>, string>;
</script>

<span
  class={cn('ui-skeleton block rounded-sm bg-muted animate-pulse-soft', shapes[shape], className)}
  style:width
  style:height={shape === 'block' || shape === 'circle' ? height : undefined}
  aria-hidden="true"
></span>
