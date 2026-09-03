import * as m from '$lib/paraglide/messages.js';
import type { CourseCategory } from './course-utils';

/**
 * The one place a course category turns into paint.
 *
 * DESIGN.md used to say category colour is applied "only through `KindBadge`",
 * which held while a badge was the only thing wearing it. The time grid draws
 * the same category as a filled block, and a block is a positioned button with
 * children — it cannot be a badge. So the rule moves down a level rather than
 * being broken: the table lives here, `KindBadge` reads it, the calendar block
 * reads it, and nothing else may spell a `--category-*` class itself.
 *
 * Every class string is a literal because Tailwind's scanner only emits what
 * it can read as one.
 */

/** Pale field plus its own readable ink. Clears 4.5:1 inside the pair and on white. */
const surfaces = {
  lecture: 'bg-category-lecture-surface text-category-lecture-text',
  tutorial: 'bg-category-tutorial-surface text-category-tutorial-text',
  lab: 'bg-category-lab-surface text-category-lab-text',
  exam: 'bg-category-exam-surface text-category-exam-text',
  project: 'bg-category-project-surface text-category-project-text',
  other: 'bg-category-other-surface text-category-other-text',
} as const satisfies Record<CourseCategory, string>;

/**
 * The saturated end of the pair, carrying white. Contrast is symmetric, so a
 * tone that clears 4.5:1 as text on white clears it again as a ground under
 * white. Reserved for the one mark a view wants to shout.
 */
const inks = {
  lecture: 'bg-category-lecture-text text-card',
  tutorial: 'bg-category-tutorial-text text-card',
  lab: 'bg-category-lab-text text-card',
  exam: 'bg-category-exam-text text-card',
  project: 'bg-category-project-text text-card',
  other: 'bg-category-other-text text-card',
} as const satisfies Record<CourseCategory, string>;

export function categorySurface(category: CourseCategory): string {
  return surfaces[category];
}

export function categoryInk(category: CourseCategory): string {
  return inks[category];
}

/**
 * A course that has already happened drops its category and takes the neutral
 * pair instead. Fading it with `opacity` was the other option and it is the
 * wrong one: opacity fades the label too, and an 11px label at 64% on a pale
 * field falls under the 4.5:1 floor. Dropping the hue also earns something —
 * nobody scans the past by category, so the palette ends up spent entirely on
 * what is still coming.
 */
export const spentSurface = 'bg-surface-sunken text-muted-foreground';

/**
 * Two or three characters, for a mark too small to carry a name. It exists so
 * the category is never communicated by hue alone, which DESIGN.md forbids and
 * which a colour-only block was quietly doing.
 */
export function categoryCode(category: CourseCategory): string {
  switch (category) {
    case 'lecture':
      return m.course_code_lecture();
    case 'tutorial':
      return m.course_code_tutorial();
    case 'lab':
      return m.course_code_lab();
    case 'exam':
      return m.course_code_exam();
    case 'project':
      return m.course_code_project();
    case 'other':
      return m.course_code_other();
  }
}
