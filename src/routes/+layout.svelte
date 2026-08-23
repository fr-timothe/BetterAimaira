<script lang="ts">
  import "../app.css";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import TitleBar from "$lib/components/TitleBar.svelte";

  let { children } = $props();

  $effect(() => {
    document.documentElement.lang = getLocale();
  });
</script>

<!-- The window edge, not an in-app card: on a desktop window that is not
     maximized the border draws the frame the OS no longer draws, and the shadow
     lifts it off the desktop, so both are needed there. -->
<div
  class="relative flex h-dvh max-h-dvh w-screen flex-col overflow-hidden bg-background
         text-foreground transition-[border-radius,border-color] duration-fast ease-out
         app-windowed:rounded-md app-windowed:border app-windowed:border-window-edge
         app-windowed:shadow-xl
         app-maximized:rounded-none app-maximized:border-0 app-maximized:shadow-none"
>
  <TitleBar />
  <div class="relative flex min-h-0 w-full flex-1 flex-col overflow-hidden">
    {@render children()}
  </div>
</div>
