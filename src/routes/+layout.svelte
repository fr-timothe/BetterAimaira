<script lang="ts">
  import { onMount } from "svelte";
  import "../app.css";
  import { initNativeInsets } from "$lib/native-insets";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import { announcer } from "$lib/state/announcements.svelte";
  import { appLocale } from "$lib/state/locale.svelte";

  let { children } = $props();

  // The safe-area insets have to be in place before the shell paints, and the
  // platform bridge only exists once the document does.
  onMount(initNativeInsets);

  $effect(() => {
    document.documentElement.lang = appLocale.current;
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
  <!-- The title bar renders its labels straight from the message catalogue, and
       Paraglide messages are not reactive. It sits outside the authenticated
       shell, so it needs its own key on the same signal to pick up a language
       change. Nothing here holds reader input, so a remount costs nothing. -->
  {#key appLocale.current}
    <TitleBar />
  {/key}
  <div class="relative flex min-h-0 w-full flex-1 flex-col overflow-hidden">
    {@render children()}
  </div>

  <!-- The application's only polite live region, and the reason it sits in the
       layout rather than in a view: a polite region has to be in the
       accessibility tree before its text changes, and this is the one node that
       survives a tab change, a locale change and the session remount. Empty most
       of the time, so its position in the reading order costs nothing. -->
  <p class="sr-only" role="status" aria-live="polite" aria-atomic="true">
    {announcer.message}
  </p>
</div>
