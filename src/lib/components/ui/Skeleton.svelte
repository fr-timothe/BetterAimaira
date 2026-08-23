<script lang="ts">
  type Props = {
    /** `text` and `title` size themselves; `block` takes the height you give it. */
    shape?: 'text' | 'title' | 'block' | 'circle';
    /** CSS length. Defaults to filling the inline axis. */
    width?: string;
    /** CSS length. Only read for `block` and `circle`. */
    height?: string;
  };

  const { shape = 'text', width, height }: Props = $props();
</script>

<span
  class="ui-skeleton {shape}"
  style:width
  style:height={shape === 'block' || shape === 'circle' ? height : undefined}
  aria-hidden="true"
></span>

<style>
  /* One skeleton grammar for the whole app: an opacity pulse on a tinted field.
     Placeholders never carry a shimmer AND a pulse. */
  .ui-skeleton {
    display: block;
    background: var(--muted);
    border-radius: var(--radius-sm);
    animation: pulse-soft 1.6s var(--ease-in-out) infinite;
  }

  .text {
    width: 100%;
    height: 0.75rem;
  }

  .title {
    width: 60%;
    height: 1.15rem;
    border-radius: var(--radius-xs);
  }

  .block {
    width: 100%;
    height: 5rem;
    border-radius: var(--radius-lg);
  }

  .circle {
    width: 2.75rem;
    height: 2.75rem;
    border-radius: 50%;
  }
</style>
