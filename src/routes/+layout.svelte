<script lang="ts">
  import "../app.css";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import TitleBar from "$lib/components/TitleBar.svelte";

  let { children } = $props();

  $effect(() => {
    document.documentElement.lang = getLocale();
  });
</script>

<div class="app-window-root">
  <TitleBar />
  <div class="app-window-content">
    {@render children()}
  </div>
</div>

<style>
  .app-window-root {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    height: 100dvh;
    max-height: 100vh;
    max-height: 100dvh;
    background: var(--background);
    color: var(--foreground);
    overflow: hidden;
    position: relative;
    box-sizing: border-box;
    transition:
      border-radius var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out);
  }

  /* Desktop window rounded corners when not maximized. This is the window edge,
     not an in-app card: the border draws the frame the OS no longer draws and
     the shadow lifts it off the desktop, so both are needed here. */
  :global(html.desktop-app:not(.window-maximized)) .app-window-root {
    border-radius: var(--radius-md);
    border: 1px solid color-mix(in oklch, var(--border) 60%, transparent);
    box-shadow: var(--shadow-xl);
  }

  :global(html.desktop-app.window-maximized) .app-window-root {
    border-radius: 0;
    border: none;
    box-shadow: none;
  }

  .app-window-content {
    display: flex;
    flex-direction: column;
    flex: 1 1 0%;
    min-height: 0;
    width: 100%;
    overflow: hidden;
    position: relative;
  }
</style>