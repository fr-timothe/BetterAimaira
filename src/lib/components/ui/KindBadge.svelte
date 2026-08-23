<script lang="ts">
  import { courseCategory, courseTypeBadge } from '$lib/features/schedule/course-utils';
  import type { CalendarEvent } from '$lib/features/schedule/types';
  import { cn } from '$lib/utils';

  type Props = {
    event: CalendarEvent;
    class?: string;
  };

  const { event, class: className }: Props = $props();

  const category = $derived(courseCategory(event.kind));
  const label = $derived(courseTypeBadge(event));

  // One badge, six category tones, all resolved from tokens — so the same
  // category shows the same colour in every view. The class names are spelled
  // out because the scanner only emits what it can read as a literal.
  const tones = {
    lecture: 'bg-category-lecture-surface text-category-lecture-text',
    tutorial: 'bg-category-tutorial-surface text-category-tutorial-text',
    lab: 'bg-category-lab-surface text-category-lab-text',
    exam: 'bg-category-exam-surface text-category-exam-text',
    project: 'bg-category-project-surface text-category-project-text',
    other: 'bg-category-other-surface text-category-other-text'
  } as const satisfies Record<ReturnType<typeof courseCategory>, string>;
</script>

<span
  class={cn(
    'ui-kind-badge inline-flex items-center rounded-xs px-2 py-[0.2rem]',
    'text-2xs font-bold whitespace-nowrap',
    tones[category],
    className
  )}>{label}</span
>
