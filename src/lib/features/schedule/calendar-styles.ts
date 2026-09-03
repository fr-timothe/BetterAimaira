/**
 * The class strings the calendar's parts share once the view was split into
 * components. They stay literal utility strings rather than becoming a scoped
 * `<style>`: DESIGN.md keeps the calendar free of one on purpose, because the
 * only numbers it computes are geometry, not style.
 */

export const panel = 'rounded-xl border border-border-subtle bg-card';

export const uppercaseTiny = 'text-xs font-bold tracking-[0.04em] uppercase';

export const monthCellBtn =
  'flex min-h-(--tap-min) w-full cursor-pointer flex-col items-center justify-center gap-[0.15rem]' +
  ' rounded-md border px-1 py-1.5 transition-control active:scale-(--press-scale)' +
  ' hover:border-primary-deep hover:bg-muted';
