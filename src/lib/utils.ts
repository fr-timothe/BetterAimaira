import { clsx, type ClassValue } from 'clsx';
import { extendTailwindMerge } from 'tailwind-merge';

/**
 * tailwind-merge only resolves conflicts between classes it recognises. This
 * system replaces Tailwind's type scale and names its own radius, duration,
 * easing and layer steps, so those have to be declared here or
 * `cn('text-sm', 'text-md')` would keep both and let Tailwind's own sort order
 * pick the winner.
 */
const twMerge = extendTailwindMerge({
  extend: {
    classGroups: {
      'font-size': [{ text: ['2xs', 'md'] }],
      rounded: [{ rounded: ['pill'] }],
      duration: [{ duration: ['instant', 'fast', 'normal', 'slow', 'spin'] }],
      ease: [{ ease: ['drawer'] }],
      z: [{ z: ['raised', 'sticky', 'nav', 'sidebar', 'overlay', 'drawer', 'modal', 'titlebar'] }]
    }
  }
});

/**
 * Compose class strings; the last conflicting utility wins.
 *
 * One ordering rule: a `leading-*` must come after the `text-*` it belongs to.
 * tailwind-merge reads `text-lg` as the size-and-leading shorthand, so a later
 * `text-*` silently drops an earlier `leading-*`.
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
