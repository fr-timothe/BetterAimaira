<script lang="ts">
  import { categorySurface } from '$lib/features/schedule/category-tone';
  import { courseCategory, courseTypeBadge } from '$lib/features/schedule/course-utils';
  import type { CalendarEvent } from '$lib/features/schedule/types';
  import { cn } from '$lib/utils';

  type Props = {
    event: CalendarEvent;
    class?: string;
  };

  const { event, class: className }: Props = $props();

  // The tone comes from `category-tone.ts` rather than a table here: the time
  // grid paints the same category as a block, and one of the two would have
  // drifted.
  const category = $derived(courseCategory(event.kind));
  const label = $derived(courseTypeBadge(event));
</script>

<span
  class={cn(
    'ui-kind-badge inline-flex items-center rounded-xs px-2 py-[0.2rem]',
    'text-2xs font-bold whitespace-nowrap',
    categorySurface(category),
    className
  )}>{label}</span
>
