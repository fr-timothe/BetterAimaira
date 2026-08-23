<script lang="ts">
  import { courseCategory, courseTypeBadge } from '$lib/features/schedule/course-utils';
  import type { CalendarEvent } from '$lib/features/schedule/types';

  type Props = {
    event: CalendarEvent;
  };

  const { event }: Props = $props();

  const category = $derived(courseCategory(event.kind));
  const label = $derived(courseTypeBadge(event));
</script>

<span class="ui-kind-badge {category}">{label}</span>

<style>
  /* One badge, six category tones, all resolved from tokens — so the same class
     shows the same colour in every view. */
  .ui-kind-badge {
    display: inline-flex;
    align-items: center;
    padding: 0.2rem var(--space-2);
    border-radius: var(--radius-xs);
    font-size: var(--text-2xs);
    font-weight: var(--weight-bold);
    line-height: 1.5;
    white-space: nowrap;
  }

  .lecture {
    color: var(--category-lecture-text);
    background: var(--category-lecture-surface);
  }

  .tutorial {
    color: var(--category-tutorial-text);
    background: var(--category-tutorial-surface);
  }

  .lab {
    color: var(--category-lab-text);
    background: var(--category-lab-surface);
  }

  .exam {
    color: var(--category-exam-text);
    background: var(--category-exam-surface);
  }

  .project {
    color: var(--category-project-text);
    background: var(--category-project-surface);
  }

  .other {
    color: var(--category-other-text);
    background: var(--category-other-surface);
  }
</style>
