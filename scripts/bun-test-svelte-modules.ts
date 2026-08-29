/**
 * Teaches `bun test` to read the `.svelte.ts` modules.
 *
 * Runes are compiler syntax, not runtime functions: outside Vite, `$state(0)`
 * is an undefined identifier and the module throws on import. Every state class
 * in `src/lib/state` is written that way, and so is the recovery logic the
 * portal cache depends on — without this, none of it is reachable from a test.
 *
 * The Svelte compiler already ships the exact entry point for this
 * (`compileModule`, the one Vite itself calls for these files), so this is
 * wiring rather than a reimplementation, and it adds no dependency. TypeScript
 * is stripped first because the compiler wants plain JavaScript.
 */
import { plugin } from 'bun';
import { readFileSync } from 'node:fs';
import { compileModule } from 'svelte/compiler';

const stripTypes = new Bun.Transpiler({ loader: 'ts' });

plugin({
  name: 'svelte-runes-modules',
  setup(build) {
    build.onLoad({ filter: /\.svelte\.(ts|js)$/ }, (args) => {
      const source = readFileSync(args.path, 'utf8');
      const javascript = args.path.endsWith('.ts')
        ? stripTypes.transformSync(source)
        : source;
      const compiled = compileModule(javascript, {
        filename: args.path,
        generate: 'client',
      });
      return { contents: compiled.js.code, loader: 'js' };
    });
  },
});
